require 'openssl'
require_relative 'runtime_helper'

class CacheGatewayRuntimeTest < CacheRuntimeTest
  def setup
    @envoy = ENV.fetch('ENVOY_BIN')
    output, error, status = CacheChart.capture(@envoy, '--version')
    assert status.success?, error
    assert_includes output, '/1.39.1/'
    super
    @gateway_port = unused_port
    @identity = 'writer'
    @server_ca = certificate('server-ca', ca: true)
    @client_ca = certificate('client-ca', ca: true)
    certificate('server', issuer: @server_ca, san: 'DNS:localhost,IP:127.0.0.1', usage: 'serverAuth')
    %w[reader writer unknown].each do |role|
      certificate(role, issuer: @client_ca, san: "URI:spiffe://oyatie.dev/build-cache/#{role}", usage: 'clientAuth')
    end
    other_ca = certificate('other-ca', ca: true)
    certificate('untrusted', issuer: other_ca, san: 'URI:spiffe://oyatie.dev/build-cache/writer', usage: 'clientAuth')
    config = YAML.safe_load(CacheChart.documents.find { |doc| doc.dig('data', 'envoy.yaml') }.dig('data', 'envoy.yaml'))
    listener = config.fetch('static_resources').fetch('listeners')[0]
    listener.fetch('address')['socket_address'] = { 'address' => '127.0.0.1', 'port_value' => @gateway_port }
    tls = listener.fetch('filter_chains')[0].dig('transport_socket', 'typed_config', 'common_tls_context')
    tls.fetch('tls_certificates')[0]['certificate_chain']['filename'] = path('server.crt')
    tls.fetch('tls_certificates')[0]['private_key']['filename'] = path('server.key')
    tls.fetch('validation_context')['trusted_ca']['filename'] = path('client-ca.crt')
    config.dig('static_resources', 'clusters', 0, 'load_assignment', 'endpoints', 0,
               'lb_endpoints', 0, 'endpoint', 'address', 'socket_address')['port_value'] = @rpc_port
    @gateway_config = path('envoy.yaml')
    File.write(@gateway_config, YAML.dump(config))
    output, error, status = CacheChart.capture(@envoy, '--mode', 'validate', '-c', @gateway_config)
    assert status.success?, "rendered Envoy configuration is invalid: #{output}\n#{error}"
    start_gateway
  end

  def teardown
    @cleanup_errors ||= []
    CacheChart.attempt_cleanup(@cleanup_errors) { stop_gateway }
    super
  end

  private

  def path(name)
    File.join(@scratch, name)
  end

  def certificate(name, ca: false, issuer: nil, san: nil, usage: nil)
    key = OpenSSL::PKey::RSA.new(2048)
    cert = OpenSSL::X509::Certificate.new
    cert.version = 2
    cert.serial = SecureRandom.random_number(2**128)
    cert.subject = OpenSSL::X509::Name.parse("/CN=#{name}")
    cert.issuer = issuer ? issuer[0].subject : cert.subject
    cert.public_key = key.public_key
    cert.not_before = Time.now - 60
    cert.not_after = Time.now + 3600
    factory = OpenSSL::X509::ExtensionFactory.new
    factory.subject_certificate = cert
    factory.issuer_certificate = issuer ? issuer[0] : cert
    cert.add_extension(factory.create_extension('basicConstraints', ca ? 'CA:TRUE' : 'CA:FALSE', true))
    cert.add_extension(factory.create_extension('keyUsage', ca ? 'keyCertSign,cRLSign' : 'digitalSignature,keyEncipherment', true))
    cert.add_extension(factory.create_extension('extendedKeyUsage', usage)) if usage
    cert.add_extension(factory.create_extension('subjectAltName', san)) if san
    cert.sign(issuer ? issuer[1] : key, OpenSSL::Digest.new('SHA256'))
    File.write(path("#{name}.crt"), cert.to_pem)
    File.write(path("#{name}.key"), key.to_pem, mode: 'w', perm: 0o600)
    [cert, key]
  end

  def start_gateway
    log = path('gateway.log')
    @gateway_pid = Process.spawn(@envoy, '-c', @gateway_config, '--concurrency', '1',
                                 '--disable-hot-restart', out: [log, 'a'], err: [:child, :out])
    Timeout.timeout(20) do
      loop do
        begin
          tls_handshake('writer')
          break
        rescue Errno::ECONNREFUSED, Errno::ECONNRESET, OpenSSL::SSL::SSLError
          sleep 0.1
        end
      end
    end
  rescue Timeout::Error
    flunk "Envoy did not become ready: #{File.read(log)}"
  end

  def stop_gateway
    CacheChart.stop(@gateway_pid)
  ensure
    @gateway_pid = nil
  end

  def tls_handshake(identity)
    context = OpenSSL::SSL::SSLContext.new
    context.min_version = OpenSSL::SSL::TLS1_2_VERSION
    context.max_version = OpenSSL::SSL::TLS1_2_VERSION
    context.ca_file = path('server-ca.crt')
    context.verify_mode = OpenSSL::SSL::VERIFY_PEER
    context.alpn_protocols = ['h2']
    unless identity == 'missing'
      context.cert = OpenSSL::X509::Certificate.new(File.read(path("#{identity}.crt")))
      context.key = OpenSSL::PKey.read(File.read(path("#{identity}.key")))
    end
    Timeout.timeout(3) do
      Socket.tcp('127.0.0.1', @gateway_port, connect_timeout: 1) do |socket|
        ssl = OpenSSL::SSL::SSLSocket.new(socket, context)
        begin
          ssl.hostname = 'localhost'
          ssl.connect
          ssl.post_connection_check('localhost')
          ssl.peer_cert.to_der
        ensure
          ssl.close
        end
      end
    end
  end

  def endpoint
    "127.0.0.1:#{@gateway_port}"
  end

  def transport_args
    ['-cacert', path('server-ca.crt'), '-cert', path("#{@identity}.crt"), '-key', path("#{@identity}.key")]
  end

  def upload_resource(digest)
    "main/uploads/#{SecureRandom.uuid}/blobs/#{digest[:hash]}/#{digest[:sizeBytes]}"
  end

  def read_stream(digest)
    output, error, status = invoke('google.bytestream.ByteStream/Read', JSON.generate(
      resourceName: "main/blobs/#{digest[:hash]}/#{digest[:sizeBytes]}"
    ))
    assert status.success?, error
    output.scan(/"data":\s*"([A-Za-z0-9+\/=]*)"/).flatten.map { |encoded| Base64.strict_decode64(encoded) }.join
  end

  def assert_denied(method, request)
    method = "build.bazel.remote.execution.v2.#{method}" unless method.start_with?('google.')
    assert_authorization_denied(*invoke(method, JSON.generate(request)))
  end

  def assert_authorization_denied(output, error, status)
    refute status.success?, 'authorization allowed a forbidden RPC'
    assert_includes error, 'PermissionDenied', "expected gateway authorization denial: #{output}\n#{error}"
  end
end
