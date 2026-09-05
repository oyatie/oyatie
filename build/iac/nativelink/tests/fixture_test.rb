require_relative 'gateway_runtime_helper'

class CacheFixtureTest < Minitest::Test
  def test_fixture_service_ports_are_distinct_when_the_os_repeats_a_candidate
    fixture = CacheRuntimeTest.new('port fixture')
    candidates = [12_345, 12_345, 23_456]
    socket_open = lambda do |*_args, &block|
      block.call(Struct.new(:addr).new([nil, candidates.shift]))
    end
    with_port_source(socket_open) do
      assert_equal [12_345, 23_456], [fixture.send(:unused_port), fixture.send(:unused_port)]
    end
  end

  def test_fixture_repeated_port_candidates_fail_with_a_bounded_error
    fixture = CacheRuntimeTest.new('port fixture')
    socket_open = ->(*_args, &block) { block.call(Struct.new(:addr).new([nil, 12_345])) }
    with_port_source(socket_open) do
      fixture.send(:unused_port)
      Timeout.timeout(1) do
        error = assert_raises(RuntimeError) { fixture.send(:unused_port) }
        assert_includes error.message, 'distinct local service port'
      end
    end
  end

  def test_fixture_gateway_failure_does_not_skip_backend_or_scratch_cleanup
    fixture = CacheGatewayRuntimeTest.new('cleanup fixture')
    scratch = Dir.mktmpdir('oyatie-cleanup-regression-')
    fixture.instance_variable_set(:@scratch, scratch)
    File.write(File.join(scratch, 'server.log'), 'retained diagnostic')
    gateway_error = RuntimeError.new('injected gateway stop failure')
    backend_stopped = false
    fixture.define_singleton_method(:stop_gateway) { raise gateway_error }
    fixture.define_singleton_method(:stop_server) { backend_stopped = true }
    error = nil
    _, logs = capture_io { error = assert_raises(StandardError) { fixture.teardown } }
    assert backend_stopped, 'gateway stop failure skipped backend stop'
    refute File.exist?(scratch), 'gateway stop failure skipped scratch removal'
    assert_includes logs, 'retained diagnostic'
    assert_equal [gateway_error], error.errors
  ensure
    FileUtils.remove_entry_secure(scratch) if scratch && File.directory?(scratch)
  end

  def test_fixture_backend_and_log_failures_preserve_errors_and_remove_scratch
    fixture = CacheRuntimeTest.new('cleanup fixture')
    scratch = Dir.mktmpdir('oyatie-cleanup-regression-')
    fixture.instance_variable_set(:@scratch, scratch)
    File.write(File.join(scratch, 'server.log'), 'retained diagnostic')
    backend_error = RuntimeError.new('injected backend stop failure')
    log_error = IOError.new('injected log reporting failure')
    fixture.define_singleton_method(:stop_server) { raise backend_error }
    fixture.define_singleton_method(:warn) { |_| raise log_error }
    error = assert_raises(StandardError) { fixture.teardown }
    refute File.exist?(scratch), 'backend stop failure skipped scratch removal'
    assert_equal [backend_error, log_error], error.errors
    assert_includes error.message, 'injected backend stop failure'
    assert_includes error.message, 'injected log reporting failure'
    assert_includes error.message, backend_error.backtrace.first
  ensure
    FileUtils.remove_entry_secure(scratch) if scratch && File.directory?(scratch)
  end

  def test_fixture_gateway_and_backend_failures_are_both_reported
    fixture = CacheGatewayRuntimeTest.new('cleanup fixture')
    gateway_error = RuntimeError.new('injected gateway stop failure')
    backend_error = RuntimeError.new('injected backend stop failure')
    fixture.define_singleton_method(:stop_gateway) { raise gateway_error }
    fixture.define_singleton_method(:stop_server) { raise backend_error }
    error = assert_raises(StandardError) { fixture.teardown }
    assert_includes error.message, 'injected backend stop failure'
    assert_includes error.message, 'injected gateway stop failure'
    assert_equal [gateway_error, backend_error], error.errors
  end

  private

  def with_port_source(source)
    original = TCPServer.method(:open)
    TCPServer.define_singleton_method(:open, source)
    yield
  ensure
    TCPServer.define_singleton_method(:open, original)
  end
end
