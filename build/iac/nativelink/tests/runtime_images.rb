require 'digest'
require 'rubygems/package'
require_relative 'helper'

module CacheRuntimeImages
  DIGEST = /\Asha256:[0-9a-f]{64}\z/
  PLATFORMS = { 'linux/amd64' => 'x86_64', 'linux/arm64' => 'aarch64' }.freeze

  def self.images(documents)
    deployments = documents.select { |document| document['kind'] == 'Deployment' }
    raise 'expected exactly one rendered cache Deployment' unless deployments.length == 1

    containers = deployments[0].fetch('spec').fetch('template').fetch('spec').fetch('containers')
    raise 'missing or ambiguous cache image source' unless containers.map { |c| c['name'] }.sort == %w[gateway nativelink]

    containers.to_h do |container|
      image = container.fetch('image')
      unless image.is_a?(String) && image.match?(%r{\A[a-z0-9][a-z0-9._/:\-]*@sha256:[0-9a-f]{64}\z})
        raise "cache image source must be immutable: #{image.inspect}"
      end
      [container.fetch('name'), container]
    end
  end

  def self.platform_digest(index, platform)
    raise "unsupported qualification platform: #{platform}" unless PLATFORMS.key?(platform)

    os, architecture = platform.split('/')
    matches = index.fetch('manifests').select do |manifest|
      manifest.dig('platform', 'os') == os && manifest.dig('platform', 'architecture') == architecture
    end
    raise "missing or ambiguous image platform: #{platform}" unless matches.length == 1

    digest = matches[0].fetch('digest')
    raise 'invalid platform image digest' unless DIGEST.match?(digest)

    digest
  end

  def self.verify_digest(reference, observed)
    expected = reference.split('@').last
    unless DIGEST.match?(expected) && observed == expected
      raise "image digest resolution diverged from immutable source: #{reference}"
    end
  end

  def self.executable_member(name, container, config, platform)
    os, architecture = platform.split('/')
    unless config['os'] == os && config['architecture'] == architecture
      raise "image configuration diverges from selected platform: #{platform}"
    end
    image_config = config.fetch('config')
    if name == 'gateway'
      unless container['command'] == ['/usr/local/bin/envoy'] &&
             image_config['Entrypoint'] == ['/docker-entrypoint.sh'] &&
             image_config['Cmd'] == ['envoy', '-c', '/etc/envoy/envoy.yaml']
        raise 'unknown gateway executable source'
      end
      return 'usr/local/bin/envoy'
    end
    entrypoint = image_config.fetch('Entrypoint')
    target = PLATFORMS.fetch(platform)
    pattern = %r{\A/nix/store/[a-z0-9]{32}-nativelink-#{target}-unknown-linux-musl-1\.6\.6/bin/nativelink\z}
    unless container['command'].nil? && entrypoint.is_a?(Array) && entrypoint.length == 1 &&
           entrypoint[0].is_a?(String) && pattern.match?(entrypoint[0])
      raise 'unknown NativeLink executable source'
    end
    entrypoint[0]
  end

  def self.extract_executable(archive, member, output)
    matches = 0
    File.open(archive, 'rb') do |file|
      Gem::Package::TarReader.new(file).each do |entry|
        next unless entry.full_name == member

        matches += 1
        unless matches == 1 && entry.file? && (entry.header.mode & 0o111).positive?
          raise "executable member is not a unique regular executable: #{member}"
        end
        File.open(output, 'wb', 0o700) { |destination| IO.copy_stream(entry, destination) }
      end
    end
    raise "missing executable image member: #{member}" unless matches == 1

    File.chmod(0o700, output)
    Digest::SHA256.file(output).hexdigest
  end

  def self.crane(*arguments)
    output, error, status = CacheChart.capture(ENV.fetch('CRANE_BIN'), *arguments, timeout: 180)
    raise "OCI acquisition failed: #{arguments.inspect}\n#{output}\n#{error}" unless status.success?

    output
  end

  def self.qualify(directory, platform)
    raise "unsupported qualification platform: #{platform}" unless PLATFORMS.key?(platform)

    images(CacheChart.documents).to_h do |name, container|
      reference = container.fetch('image')
      repository = reference.split('@').first
      verify_digest(reference, crane('digest', reference).strip)

      index = JSON.parse(crane('manifest', reference))
      digest = platform_digest(index, platform)
      resolved = "#{repository}@#{digest}"
      verify_digest(resolved, crane('digest', resolved).strip)

      config = JSON.parse(crane('config', resolved))
      member = executable_member(name, container, config, platform)
      output = File.join(directory, name == 'gateway' ? 'envoy' : 'nativelink')
      hash = Dir.mktmpdir('oyatie-cache-image-') do |scratch|
        archive = File.join(scratch, 'image.tar')
        crane('export', resolved, archive)
        extract_executable(archive, member, output)
      end
      version, error, status = CacheChart.capture(output, '--version')
      expected_version = name == 'gateway' ? '/1.39.1/' : '1.6.6'
      raise "unexpected image executable version: #{version}\n#{error}" unless status.success? && version.include?(expected_version)

      puts JSON.generate(name: name, chart_image: reference, platform: platform,
                         platform_image: resolved, executable_member: member,
                         executable_sha256: hash, executable_version: version.strip)
      [name, output]
    end
  end
end

if $PROGRAM_NAME == __FILE__
  binaries = CacheRuntimeImages.qualify(ENV.fetch('CACHE_RUNTIME_DIR'), ENV.fetch('CACHE_PLATFORM'))
  File.open(ENV.fetch('GITHUB_ENV'), 'a') do |environment|
    environment.puts "NATIVELINK_BIN=#{binaries.fetch('nativelink')}"
    environment.puts "ENVOY_BIN=#{binaries.fetch('gateway')}"
  end
end
