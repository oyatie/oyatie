require 'json'
require 'open3'
require 'yaml'

module CacheChart
  CHART = File.expand_path('..', __dir__)
  PLACEMENT = {
    'nodeHostname' => 'qualification-node',
    'existingClaim' => 'qualification-cache',
    'existingTlsSecret' => 'qualification-tls',
    'certificateRevision' => 'qualification'
  }.freeze

  def self.render(overrides = {})
    args = ['helm', 'template', 'build-cache', CHART]
    PLACEMENT.merge(overrides).each { |key, value| args.concat(['--set', "#{key}=#{value}"]) }
    Open3.capture3(*args)
  end

  def self.documents
    output, error, status = render
    raise "chart rendering failed: #{error}" unless status.success?

    YAML.load_stream(output).compact
  end

  def self.native_config
    document = documents.find { |doc| doc.dig('data', 'config.json') }
    JSON.parse(document.fetch('data').fetch('config.json'))
  end
end
