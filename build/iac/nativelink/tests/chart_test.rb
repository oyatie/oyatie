require 'minitest/autorun'
require_relative 'helper'

class CacheChartTest < Minitest::Test
  def setup
    @documents = CacheChart.documents
    @deployment = @documents.find { |doc| doc['kind'] == 'Deployment' }
    @pod = @deployment.fetch('spec').fetch('template').fetch('spec')
  end

  def test_missing_placement_or_credentials_prevents_rendering
    CacheChart::PLACEMENT.each_key do |key|
      _, error, status = CacheChart.render(key => '')
      refute status.success?, "missing #{key} was admitted"
      assert_includes error, key
    end
  end

  def test_unbounded_or_invalid_cache_settings_are_rejected
    {
      'cache.casBytes' => 36_507_222_017,
      'cache.casObjects' => 0,
      'cache.acBytes' => 67_108_864,
      'cache.acObjects' => 50_001,
      'instanceName' => 'unqualified',
      'service.type' => 'LoadBalancer'
    }.each do |key, value|
      _, _, status = CacheChart.render(key => value)
      refute status.success?, "invalid #{key}=#{value} was admitted"
    end
  end

  def test_cache_requires_existing_volume_and_cannot_expose_raw_backend
    assert_equal 1, @deployment.dig('spec', 'replicas')
    assert_equal 'Recreate', @deployment.dig('spec', 'strategy', 'type')
    assert_equal 'qualification-node', @pod.dig('nodeSelector', 'kubernetes.io/hostname')
    assert_equal 'arm64', @pod.dig('nodeSelector', 'kubernetes.io/arch')
    assert_equal %w[ConfigMap ConfigMap Deployment NetworkPolicy Service], @documents.map { |d| d['kind'] }.sort
    cache = @pod.fetch('volumes').find { |volume| volume['name'] == 'cache' }
    assert_equal({'claimName' => 'qualification-cache'}, cache.fetch('persistentVolumeClaim'))
    refute @pod.fetch('volumes').any? { |volume| volume.key?('hostPath') || volume.key?('emptyDir') }
    service = @documents.find { |doc| doc['kind'] == 'Service' }
    assert_equal 'ClusterIP', service.dig('spec', 'type')
    assert_equal [443], service.dig('spec', 'ports').map { |port| port['port'] }
    assert_equal ['grpcs'], service.dig('spec', 'ports').map { |port| port['targetPort'] }
    network = @documents.find { |doc| doc['kind'] == 'NetworkPolicy' }.fetch('spec')
    assert_equal [], network.fetch('egress')
    assert_equal [8443], network.fetch('ingress').flat_map { |rule| rule.fetch('ports').map { |p| p['port'] } }
  end

  def test_runtime_privileges_and_resource_limits_are_bounded
    assert_equal false, @pod.fetch('automountServiceAccountToken')
    assert_equal true, @pod.dig('securityContext', 'runAsNonRoot')
    assert_equal 65532, @pod.dig('securityContext', 'runAsUser')
    assert_equal %w[nativelink gateway], @pod.fetch('containers').map { |c| c['name'] }
    @pod.fetch('containers').each do |container|
      assert_match(/@sha256:[0-9a-f]{64}\z/, container.fetch('image'))
      assert_equal false, container.dig('securityContext', 'allowPrivilegeEscalation')
      assert_equal true, container.dig('securityContext', 'readOnlyRootFilesystem')
      assert_equal ['ALL'], container.dig('securityContext', 'capabilities', 'drop')
    end
    assert_equal [{'cpu' => '750m', 'memory' => '3Gi'}, {'cpu' => '250m', 'memory' => '512Mi'}],
                 @pod.fetch('containers').map { |c| c.dig('resources', 'limits') }
  end

  def test_native_services_and_storage_validation
    config = CacheChart.native_config
    server = config.fetch('servers').find { |item| item['name'] == 'cache' }
    assert_equal '127.0.0.1:50051', server.dig('listener', 'http', 'socket_address')
    assert_equal %w[ac bytestream capabilities cas], server.fetch('services').keys.sort
    assert_equal ['main'], server.fetch('services').values.flatten.map { |s| s['instance_name'] }.uniq
    stores = config.fetch('stores').to_h { |store| [store.fetch('name'), store] }
    assert_equal true, stores.dig('cas', 'verify', 'verify_hash')
    assert_equal true, stores.dig('cas', 'verify', 'verify_size')
    assert_equal 'cas', stores.dig('ac', 'completeness_checking', 'cas_store', 'ref_store', 'name')
    %w[cas ac].each do |name|
      fs = stores.fetch("#{name}-files").fetch('filesystem')
      assert_equal "/cache/#{name}", fs.fetch('content_path')
      assert_equal "/cache/tmp/#{name}", fs.fetch('temp_path')
      assert_equal 2, fs.fetch('max_concurrent_writes')
      assert_operator fs.dig('eviction_policy', 'max_count'), :>, 0
    end
  end

  def test_mtls_roles_authorize_only_expected_post_methods
    config = YAML.safe_load(@documents.find { |doc| doc.dig('data', 'envoy.yaml') }.dig('data', 'envoy.yaml'))
    chain = config.dig('static_resources', 'listeners', 0, 'filter_chains', 0)
    tls = chain.dig('transport_socket', 'typed_config')
    assert_equal true, tls.fetch('require_client_certificate')
    assert_equal '/tls/ca.crt', tls.dig('common_tls_context', 'validation_context', 'trusted_ca', 'filename')
    filters = chain.fetch('filters')
    assert_equal 32, filters.find { |f| f['name'] == 'envoy.filters.network.connection_limit' }.dig('typed_config', 'max_connections')
    http = filters.find { |f| f['name'] == 'envoy.filters.network.http_connection_manager' }.fetch('typed_config')
    rules = http.fetch('http_filters').find { |f| f['name'] == 'envoy.filters.http.rbac' }.dig('typed_config', 'rules')
    assert_equal 'ALLOW', rules.fetch('action')
    policies = rules.fetch('policies')
    assert_equal %w[read write], policies.keys.sort
    assert_equal %w[reader writer], policies.fetch('read').fetch('principals').map { |p| role(p) }
    assert_equal ['writer'], policies.fetch('write').fetch('principals').map { |p| role(p) }
    read_methods = allowed_paths(policies.fetch('read'))
    write_methods = allowed_paths(policies.fetch('write'))
    assert_equal %w[GetCapabilities FindMissingBlobs BatchReadBlobs GetTree GetActionResult Read].sort,
                 read_methods.map { |path| path.split('/').last }.sort
    assert_equal %w[BatchUpdateBlobs UpdateActionResult Write QueryWriteStatus].sort,
                 write_methods.map { |path| path.split('/').last }.sort
    refute (read_methods + write_methods).any? { |path| path.include?('Execution') || path.include?('Worker') }
  end

  def test_certificate_revision_changes_pod_template
    output, _, status = CacheChart.render('certificateRevision' => 'rotation')
    assert status.success?
    changed = YAML.load_stream(output).compact.find { |doc| doc['kind'] == 'Deployment' }
    refute_equal @deployment.dig('spec', 'template'), changed.dig('spec', 'template')
  end

  private

  def role(principal)
    custom = principal.fetch('custom')
    assert_equal 'envoy.rbac.principals.mtls_authenticated', custom.fetch('name')
    matcher = custom.dig('typed_config', 'san_matcher')
    assert_equal 'URI', matcher.fetch('san_type')
    value = matcher.dig('matcher', 'exact')
    assert_match(%r{\Aspiffe://oyatie\.dev/build-cache/(reader|writer)\z}, value)
    value.split('/').last
  end

  def allowed_paths(policy)
    rules = policy.dig('permissions', 0, 'and_rules', 'rules')
    assert_equal({ 'name' => ':method', 'string_match' => { 'exact' => 'POST' } }, rules[0].fetch('header'))
    rules[1].dig('or_rules', 'rules').map { |permission| permission.fetch('url_path').fetch('path').fetch('exact') }
  end
end
