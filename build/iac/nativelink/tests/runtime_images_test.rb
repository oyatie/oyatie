require 'minitest/autorun'
require_relative 'runtime_images'

class CacheRuntimeImagesTest < Minitest::Test
  def test_image_input_changes_with_the_rendered_chart
    documents = CacheChart.documents
    original = CacheRuntimeImages.images(documents).fetch('nativelink').fetch('image')
    container = documents.find { |doc| doc['kind'] == 'Deployment' }.dig('spec', 'template', 'spec', 'containers')[0]
    changed = "ghcr.io/tracemachina/nativelink@sha256:#{'1' * 64}"
    refute_equal original, changed
    container['image'] = changed
    assert_equal changed, CacheRuntimeImages.images(documents).fetch('nativelink').fetch('image')
  end

  def test_missing_mutable_and_ambiguous_chart_images_are_refused
    documents = CacheChart.documents
    deployment = documents.find { |doc| doc['kind'] == 'Deployment' }
    containers = deployment.dig('spec', 'template', 'spec', 'containers')
    containers[0]['image'] = 'ghcr.io/tracemachina/nativelink:latest'
    assert_raises(RuntimeError) { CacheRuntimeImages.images(documents) }
    containers.pop
    assert_raises(RuntimeError) { CacheRuntimeImages.images(documents) }
    assert_raises(RuntimeError) { CacheRuntimeImages.images([deployment, deployment]) }
    assert_raises(RuntimeError) { CacheRuntimeImages.images([]) }
  end

  def test_platform_selection_is_exact_and_refuses_missing_or_ambiguous_sources
    arm = { 'digest' => "sha256:#{'a' * 64}", 'platform' => { 'os' => 'linux', 'architecture' => 'arm64' } }
    amd = { 'digest' => "sha256:#{'b' * 64}", 'platform' => { 'os' => 'linux', 'architecture' => 'amd64' } }
    assert_equal "sha256:#{'a' * 64}", CacheRuntimeImages.platform_digest({ 'manifests' => [arm, amd] }, 'linux/arm64')
    assert_raises(RuntimeError) { CacheRuntimeImages.platform_digest({ 'manifests' => [amd] }, 'linux/arm64') }
    assert_raises(RuntimeError) { CacheRuntimeImages.platform_digest({ 'manifests' => [arm, arm] }, 'linux/arm64') }
    arm['digest'] = 'mutable'
    assert_raises(RuntimeError) { CacheRuntimeImages.platform_digest({ 'manifests' => [arm] }, 'linux/arm64') }
  end

  def test_resolved_digest_must_match_the_requested_immutable_source
    reference = "registry.example/cache@sha256:#{'a' * 64}"
    CacheRuntimeImages.verify_digest(reference, "sha256:#{'a' * 64}")
    ["sha256:#{'b' * 64}", '', 'latest'].each do |observed|
      assert_raises(RuntimeError) { CacheRuntimeImages.verify_digest(reference, observed) }
    end
  end

  def test_configuration_cannot_select_a_divergent_executable_or_architecture
    member = "/nix/store/#{'a' * 32}-nativelink-aarch64-unknown-linux-musl-1.6.6/bin/nativelink"
    config = { 'os' => 'linux', 'architecture' => 'arm64', 'config' => { 'Entrypoint' => [member] } }
    assert_equal member, CacheRuntimeImages.executable_member('nativelink', {}, config, 'linux/arm64')
    assert_raises(RuntimeError) do
      CacheRuntimeImages.executable_member('nativelink', { 'command' => ['/bin/sh'] }, config, 'linux/arm64')
    end
    config['architecture'] = 'amd64'
    assert_raises(RuntimeError) { CacheRuntimeImages.executable_member('nativelink', {}, config, 'linux/arm64') }
    config['architecture'] = 'arm64'
    ['/bin/nativelink', '/../../bin/nativelink', nil].each do |entrypoint|
      config['config']['Entrypoint'] = [entrypoint]
      assert_raises(RuntimeError) { CacheRuntimeImages.executable_member('nativelink', {}, config, 'linux/arm64') }
    end
    gateway = { 'command' => ['/usr/local/bin/envoy'] }
    config['config'] = { 'Entrypoint' => ['/docker-entrypoint.sh'], 'Cmd' => ['envoy', '-c', '/etc/envoy/envoy.yaml'] }
    assert_equal 'usr/local/bin/envoy', CacheRuntimeImages.executable_member('gateway', gateway, config, 'linux/arm64')
    gateway['command'] = ['/docker-entrypoint.sh']
    assert_raises(RuntimeError) { CacheRuntimeImages.executable_member('gateway', gateway, config, 'linux/arm64') }
  end

  def test_only_the_exact_regular_executable_member_is_copied
    Dir.mktmpdir('oyatie-image-regression-') do |scratch|
      archive, output = %w[image.tar binary].map { |file| File.join(scratch, file) }
      member = '/nix/store/exact/bin/nativelink'
      tar(archive) do |writer|
        writer.add_symlink('/bin/nativelink', member, 0o777)
        writer.add_file_simple(member, 0o555, 5) { |entry| entry.write('hello') }
      end
      hash = CacheRuntimeImages.extract_executable(archive, member, output)
      assert_equal 'hello', File.binread(output)
      assert_equal '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824', hash
      assert File.executable?(output)
    end
  end

  def test_missing_symlink_duplicate_and_nonexecutable_members_are_refused
    Dir.mktmpdir('oyatie-image-regression-') do |scratch|
      archive, output = %w[image.tar binary].map { |file| File.join(scratch, file) }
      member = 'usr/local/bin/envoy'
      tar(archive) { |_| }
      assert_raises(RuntimeError) { CacheRuntimeImages.extract_executable(archive, member, output) }
      tar(archive) { |writer| writer.add_symlink(member, '../../outside', 0o777) }
      assert_raises(RuntimeError) { CacheRuntimeImages.extract_executable(archive, member, output) }
      tar(archive) { |writer| 2.times { writer.add_file_simple(member, 0o755, 0) {} } }
      assert_raises(RuntimeError) { CacheRuntimeImages.extract_executable(archive, member, output) }
      tar(archive) { |writer| writer.add_file_simple(member, 0o644, 0) {} }
      assert_raises(RuntimeError) { CacheRuntimeImages.extract_executable(archive, member, output) }
    end
  end

  private

  def tar(path)
    File.open(path, 'wb') { |file| Gem::Package::TarWriter.new(file) { |writer| yield writer } }
  end
end
