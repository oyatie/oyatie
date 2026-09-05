require 'base64'
require 'digest'
require 'fileutils'
require 'minitest/autorun'
require 'net/http'
require 'securerandom'
require 'socket'
require 'timeout'
require 'tmpdir'
require_relative 'helper'

class CacheStoreTest < Minitest::Test
  def setup
    @native = ENV.fetch('NATIVELINK_BIN')
    @grpcurl = ENV.fetch('GRPCURL_BIN')
    @remote_apis = ENV.fetch('REMOTE_APIS_DIR')
    @googleapis = ENV.fetch('GOOGLEAPIS_DIR')
    version, error, status = Open3.capture3(@native, '--version')
    assert status.success?, error
    assert_includes version, '1.6.6'
    @scratch = Dir.mktmpdir('oyatie-cache-store-')
    @rpc_port = unused_port
    @health_port = unused_port
    config = CacheChart.native_config
    config.fetch('stores').each do |store|
      next unless store.key?('filesystem')

      filesystem = store.fetch('filesystem')
      %w[content_path temp_path].each do |key|
        filesystem[key] = filesystem.fetch(key).sub('/cache', @scratch)
      end
      filesystem.fetch('eviction_policy')['max_count'] = 3
    end
    config.fetch('servers')[0].fetch('listener').fetch('http')['socket_address'] = "127.0.0.1:#{@rpc_port}"
    config.fetch('servers')[1].fetch('listener').fetch('http')['socket_address'] = "127.0.0.1:#{@health_port}"
    @config_path = File.join(@scratch, 'config.json')
    File.write(@config_path, JSON.generate(config))
    start_server
  end

  def teardown
    stop_server
    FileUtils.remove_entry_secure(@scratch) if @scratch && File.directory?(@scratch)
  end

  def test_real_store_integrity_streaming_restart_and_stale_action_rejection
    capabilities = rpc('Capabilities/GetCapabilities', instanceName: 'main')
    refute capabilities.dig('executionCapabilities', 'execEnabled')
    blob = 'persistent qualification blob'
    digest = blob_digest(blob)
    write_blob(blob)
    assert_equal blob, read_blob(digest)

    rejected = rpc('ContentAddressableStorage/BatchUpdateBlobs',
                   instanceName: 'main', requests: [{ digest: { hash: '0' * 64, sizeBytes: blob.bytesize }, data: Base64.strict_encode64(blob) }])
    refute_equal 0, rejected.fetch('responses')[0].fetch('status').fetch('code', 0)

    streamed = 'streamed-cache-data-' * 350_000
    stream_digest = blob_digest(streamed)
    resource = "main/uploads/#{SecureRandom.uuid}/blobs/#{stream_digest[:hash]}/#{stream_digest[:sizeBytes]}"
    chunks = streamed.bytes.each_slice(524_288).map.with_index do |bytes, index|
      { resourceName: resource, writeOffset: index * 524_288, data: Base64.strict_encode64(bytes.pack('C*')) }
    end
    chunks.last[:finishWrite] = true
    output, error, status = invoke('google.bytestream.ByteStream/Write', chunks.map { |chunk| JSON.generate(chunk) }.join("\n"))
    assert status.success?, error
    assert_equal streamed.bytesize, JSON.parse(output).fetch('committedSize').to_i
    output, error, status = invoke('google.bytestream.ByteStream/Read', JSON.generate(resourceName: "main/blobs/#{stream_digest[:hash]}/#{stream_digest[:sizeBytes]}"))
    assert status.success?, error
    actual = output.scan(/"data":\s*"([A-Za-z0-9+\/=]*)"/).flatten.map { |encoded| Base64.strict_decode64(encoded) }.join
    assert_equal streamed, actual

    stop_server
    start_server
    assert_equal blob, read_blob(digest)

    action = blob_digest('qualification action')
    result = { outputFiles: [{ path: 'artifact', digest: digest, isExecutable: false }] }
    rpc('ActionCache/UpdateActionResult', instanceName: 'main', actionDigest: action, actionResult: result)
    cached = rpc('ActionCache/GetActionResult', instanceName: 'main', actionDigest: action)
    assert_equal digest[:hash], cached.fetch('outputFiles')[0].fetch('digest').fetch('hash')

    missing_action = blob_digest('missing-output action')
    missing_result = { outputFiles: [{ path: 'missing', digest: blob_digest('never uploaded') }] }
    rpc('ActionCache/UpdateActionResult', instanceName: 'main', actionDigest: missing_action, actionResult: missing_result)
    _, error, status = invoke('build.bazel.remote.execution.v2.ActionCache/GetActionResult', JSON.generate(instanceName: 'main', actionDigest: missing_action))
    refute status.success?, 'stale action was returned as a cache hit'
    assert_includes error, 'NotFound'

    5.times { |index| write_blob("eviction object #{index}") }
    missing = rpc('ContentAddressableStorage/FindMissingBlobs', instanceName: 'main', blobDigests: [digest])
    assert_equal digest[:hash], missing.fetch('missingBlobDigests')[0].fetch('hash')
    _, error, status = invoke('build.bazel.remote.execution.v2.ActionCache/GetActionResult', JSON.generate(instanceName: 'main', actionDigest: action))
    refute status.success?, 'an action referencing evicted CAS data remained a hit'
    assert_includes error, 'NotFound'
  end

  private

  def unused_port
    TCPServer.open('127.0.0.1', 0) { |socket| socket.addr[1] }
  end

  def start_server
    log_path = File.join(@scratch, 'server.log')
    @pid = Process.spawn(@native, @config_path, out: log_path, err: [:child, :out])
    Timeout.timeout(20) do
      loop do
        begin
          response = Net::HTTP.start('127.0.0.1', @health_port, nil, open_timeout: 1, read_timeout: 1) { |http| http.get('/status') }
          break if response.code == '200'
        rescue Errno::ECONNREFUSED, Errno::ECONNRESET, EOFError, Net::ReadTimeout
          # Only bounded local readiness polling; no remote endpoints are contacted.
        end
        sleep 0.1
      end
    end
  rescue Timeout::Error
    flunk "NativeLink did not become ready: #{File.read(log_path)}"
  end

  def stop_server
    return unless @pid

    Process.kill('TERM', @pid)
    Timeout.timeout(5) { Process.wait(@pid) }
  rescue Errno::ESRCH, Errno::ECHILD
    nil
  rescue Timeout::Error
    Process.kill('KILL', @pid)
    Process.wait(@pid)
  ensure
    @pid = nil
  end

  def blob_digest(data)
    { hash: Digest::SHA256.hexdigest(data), sizeBytes: data.bytesize }
  end

  def write_blob(data)
    response = rpc('ContentAddressableStorage/BatchUpdateBlobs',
                   instanceName: 'main', requests: [{ digest: blob_digest(data), data: Base64.strict_encode64(data) }])
    assert_equal 0, response.fetch('responses')[0].fetch('status').fetch('code', 0)
  end

  def read_blob(digest)
    response = rpc('ContentAddressableStorage/BatchReadBlobs', instanceName: 'main', digests: [digest])
    entry = response.fetch('responses')[0]
    assert_equal 0, entry.fetch('status').fetch('code', 0)
    Base64.strict_decode64(entry.fetch('data'))
  end

  def rpc(method, request)
    output, error, status = invoke("build.bazel.remote.execution.v2.#{method}", JSON.generate(request))
    assert status.success?, error
    JSON.parse(output)
  end

  def invoke(method, input)
    proto = method.start_with?('google.bytestream.') ? 'google/bytestream/bytestream.proto' : 'build/bazel/remote/execution/v2/remote_execution.proto'
    Open3.capture3(@grpcurl, '-plaintext', '-max-time', '15',
                   '-import-path', @remote_apis, '-import-path', @googleapis,
                   '-proto', proto, '-d', '@', "127.0.0.1:#{@rpc_port}", method, stdin_data: input)
  end
end
