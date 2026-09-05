require_relative 'gateway_runtime_helper'

class CacheGatewayTest < CacheGatewayRuntimeTest
  def test_writer_and_reader_cache_access_survives_process_and_serving_certificate_replacement
    blob = 'gateway retained CAS'
    digest = blob_digest(blob)
    write_blob(blob)
    assert_equal blob, read_blob(digest)
    streamed = 'gateway streamed data' * 32_768
    stream_digest = blob_digest(streamed)
    resource = upload_resource(stream_digest)
    output, error, status = invoke('google.bytestream.ByteStream/Write', JSON.generate(
      resourceName: resource, data: Base64.strict_encode64(streamed), finishWrite: true
    ))
    assert status.success?, error
    assert_equal streamed.bytesize, JSON.parse(output).fetch('committedSize').to_i
    assert_equal streamed, read_stream(stream_digest)
    @identity = 'reader'
    assert_equal blob, read_blob(digest)
    assert_equal streamed, read_stream(stream_digest)
    stop_server
    start_server
    assert_equal blob, read_blob(digest)
    assert_equal streamed, read_stream(stream_digest)
    old_leaf = tls_handshake('reader')
    stop_gateway
    certificate('server', issuer: @server_ca, san: 'DNS:localhost,IP:127.0.0.1', usage: 'serverAuth')
    start_gateway
    replacement_leaf = tls_handshake('reader')
    refute_equal old_leaf, replacement_leaf
    assert_equal OpenSSL::X509::Certificate.new(File.read(path('server.crt'))).to_der, replacement_leaf
    assert_equal blob, read_blob(digest)
    assert_equal streamed, read_stream(stream_digest)
  end

  def test_reader_write_denials_leave_cas_and_action_cache_unchanged
    retained = 'retained authorized blob'
    write_blob(retained)
    action = blob_digest('authorized action')
    rpc('ActionCache/UpdateActionResult', instanceName: 'main', actionDigest: action, actionResult: { exitCode: 7 })
    candidate = 'reader must never store this'
    digest = blob_digest(candidate)
    @identity = 'reader'
    assert_denied('ContentAddressableStorage/BatchUpdateBlobs', instanceName: 'main',
                  requests: [{ digest: digest, data: Base64.strict_encode64(candidate) }])
    assert_denied('ActionCache/UpdateActionResult', instanceName: 'main', actionDigest: action, actionResult: { exitCode: 9 })
    resource = upload_resource(digest)
    assert_denied('google.bytestream.ByteStream/Write', resourceName: resource, data: Base64.strict_encode64(candidate), finishWrite: true)
    assert_denied('google.bytestream.ByteStream/QueryWriteStatus', resourceName: resource)
    missing = rpc('ContentAddressableStorage/FindMissingBlobs', instanceName: 'main', blobDigests: [digest])
    assert_equal [digest[:hash]], missing.fetch('missingBlobDigests').map { |entry| entry.fetch('hash') }
    assert_equal retained, read_blob(blob_digest(retained))
    assert_equal 7, rpc('ActionCache/GetActionResult', instanceName: 'main', actionDigest: action).fetch('exitCode')
  end

  def test_tls_trust_and_rpc_allowlist_are_enforced
    %w[missing untrusted].each do |identity|
      error = assert_raises(OpenSSL::SSL::SSLError) { tls_handshake(identity) }
      assert_match(/alert|certificate|handshake/i, error.message)
    end
    @identity = 'unknown'
    assert_denied('Capabilities/GetCapabilities', instanceName: 'main')
    @identity = 'writer'
    assert_denied('Execution/Execute', instanceName: 'main', actionDigest: blob_digest('unavailable execution'))
    File.write(path('unknown.proto'), 'syntax = "proto3"; package unknown; message Empty {} service Service { rpc Call(Empty) returns (Empty); }')
    output, error, status = CacheChart.capture(@grpcurl, *transport_args, '-max-time', '15',
      '-import-path', @scratch, '-proto', 'unknown.proto', '-d', '{}', endpoint, 'unknown.Service/Call')
    assert_authorization_denied(output, error, status)
  end

  def test_denial_assertion_detects_a_reader_granted_write_permission
    stop_gateway
    config = YAML.safe_load(File.read(@gateway_config))
    filters = config.dig('static_resources', 'listeners', 0, 'filter_chains', 0, 'filters')
    manager = filters.find { |filter| filter['name'] == 'envoy.filters.network.http_connection_manager' }
    rbac = manager.dig('typed_config', 'http_filters').find { |filter| filter['name'] == 'envoy.filters.http.rbac' }
    writer = rbac.dig('typed_config', 'rules', 'policies', 'write', 'principals', 0)
    writer.dig('custom', 'typed_config', 'san_matcher', 'matcher')['exact'] = 'spiffe://oyatie.dev/build-cache/reader'
    File.write(@gateway_config, YAML.dump(config))
    start_gateway
    @identity = 'reader'
    blob = 'mutation witness'
    failure = assert_raises(Minitest::Assertion) do
      assert_denied('ContentAddressableStorage/BatchUpdateBlobs', instanceName: 'main',
                    requests: [{ digest: blob_digest(blob), data: Base64.strict_encode64(blob) }])
    end
    assert_includes failure.message, 'authorization allowed a forbidden RPC'
    assert_equal blob, read_blob(blob_digest(blob))
  end

end
