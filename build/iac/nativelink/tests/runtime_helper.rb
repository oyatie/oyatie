require 'base64'
require 'digest'
require 'fileutils'
require 'minitest/autorun'
require 'net/http'
require 'securerandom'
require 'socket'
require_relative 'helper'

class CacheRuntimeTest < Minitest::Test
  def setup
    @native = ENV.fetch('NATIVELINK_BIN')
    @grpcurl = ENV.fetch('GRPCURL_BIN')
    @remote_apis = ENV.fetch('REMOTE_APIS_DIR')
    @googleapis = ENV.fetch('GOOGLEAPIS_DIR')
    output, error, status = CacheChart.capture(@native, '--version')
    assert status.success?, error
    assert_includes output, '1.6.6'
    @scratch = Dir.mktmpdir('oyatie-cache-runtime-')
    @rpc_port, @health_port = unused_port, unused_port
    config = CacheChart.native_config
    config.fetch('stores').each do |store|
      next unless store.key?('filesystem')

      filesystem = store.fetch('filesystem')
      %w[content_path temp_path].each do |key|
        filesystem[key] = filesystem.fetch(key).sub('/cache', @scratch)
      end
      filesystem.fetch('eviction_policy')['max_count'] = object_limit if object_limit
    end
    config.fetch('servers')[0].fetch('listener').fetch('http')['socket_address'] = "127.0.0.1:#{@rpc_port}"
    config.fetch('servers')[1].fetch('listener').fetch('http')['socket_address'] = "127.0.0.1:#{@health_port}"
    @config_path = File.join(@scratch, 'config.json')
    File.write(@config_path, JSON.generate(config))
    start_server
  end

  def teardown
    @cleanup_errors ||= []
    CacheChart.attempt_cleanup(@cleanup_errors) { stop_server }
    CacheChart.attempt_cleanup(@cleanup_errors) do
      if @scratch && File.directory?(@scratch) && (!passed? || !@cleanup_errors.empty?)
        Dir.glob(File.join(@scratch, '*.log')).each { |path| warn "#{File.basename(path)}:\n#{File.read(path)}" }
      end
    end
    CacheChart.attempt_cleanup(@cleanup_errors) do
      FileUtils.remove_entry_secure(@scratch) if @scratch && File.directory?(@scratch)
    end
    raise CacheChart::CleanupError.new(@cleanup_errors) unless @cleanup_errors.empty?
  end

  private

  def object_limit
    nil
  end

  def unused_port
    @allocated_ports ||= []
    20.times do
      port = TCPServer.open('127.0.0.1', 0) { |socket| socket.addr[1] }
      next if @allocated_ports.include?(port)

      @allocated_ports << port
      return port
    end
    raise 'could not allocate a distinct local service port after 20 candidates'
  end

  def start_server
    log_path = File.join(@scratch, 'server.log')
    @pid = Process.spawn(@native, @config_path, out: [log_path, 'a'], err: [:child, :out])
    Timeout.timeout(20) do
      loop do
        begin
          response = Net::HTTP.start('127.0.0.1', @health_port, nil, open_timeout: 1, read_timeout: 1) { |http| http.get('/status') }
          break if response.code == '200'
        rescue Errno::ECONNREFUSED, Errno::ECONNRESET, EOFError, Net::ReadTimeout
          # Bounded loopback readiness polling.
        end
        sleep 0.1
      end
    end
  rescue Timeout::Error
    flunk "NativeLink did not become ready: #{File.read(log_path)}"
  end

  def stop_server
    CacheChart.stop(@pid)
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
    assert status.success?, "#{method}: #{output}\n#{error}"
    JSON.parse(output)
  end

  def transport_args
    ['-plaintext']
  end

  def endpoint
    "127.0.0.1:#{@rpc_port}"
  end

  def invoke(method, input)
    proto = method.start_with?('google.bytestream.') ? 'google/bytestream/bytestream.proto' : 'build/bazel/remote/execution/v2/remote_execution.proto'
    CacheChart.capture(@grpcurl, *transport_args, '-max-time', '15',
                       '-import-path', @remote_apis, '-import-path', @googleapis,
                       '-proto', proto, '-d', '@', endpoint, method, stdin_data: input)
  end
end
