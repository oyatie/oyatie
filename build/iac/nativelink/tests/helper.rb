require 'json'
require 'open3'
require 'yaml'
require 'tmpdir'
require 'timeout'

module CacheChart
  class CleanupError < StandardError
    attr_reader :errors

    def initialize(errors)
      @errors = errors.freeze
      super(errors.map { |error| error.full_message(highlight: false) }.join("\n"))
    end
  end

  def self.attempt_cleanup(errors)
    yield
  rescue StandardError => error
    errors << error
  end

  CHART = File.expand_path('..', __dir__)
  PLACEMENT = {
    'nodeHostname' => 'qualification-node',
    'existingClaim' => 'qualification-cache',
    'existingTlsSecret' => 'qualification-tls',
    'certificateRevision' => 'qualification'
  }.freeze

  def self.capture(*args, stdin_data: '', timeout: 30)
    Dir.mktmpdir('oyatie-cache-command-') do |scratch|
      input, output, error = %w[in out err].map { |name| File.join(scratch, name) }
      File.write(input, stdin_data)
      pid = Process.spawn(*args, in: input, out: output, err: error)
      begin
        _, status = Timeout.timeout(timeout) { Process.wait2(pid) }
      rescue Timeout::Error
        stop(pid)
        raise "command timed out: #{args.inspect}\n#{File.read(output)}\n#{File.read(error)}"
      end
      [File.read(output), File.read(error), status]
    end
  end

  def self.stop(pid)
    return unless pid

    Process.kill('TERM', pid)
    Timeout.timeout(5) { Process.wait(pid) }
  rescue Errno::ESRCH, Errno::ECHILD
    nil
  rescue Timeout::Error
    Process.kill('KILL', pid)
    Timeout.timeout(5) { Process.wait(pid) }
  end

  def self.render(overrides = {})
    args = ['helm', 'template', 'build-cache', CHART]
    PLACEMENT.merge(overrides).each { |key, value| args.concat(['--set', "#{key}=#{value}"]) }
    capture(*args)
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
