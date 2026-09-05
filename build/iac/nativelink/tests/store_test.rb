require_relative 'runtime_helper'

class CacheStoreTest < CacheRuntimeTest
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

  def object_limit
    3
  end
end
