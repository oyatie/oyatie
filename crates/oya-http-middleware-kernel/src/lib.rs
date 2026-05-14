//! HTTP middleware kernel — pure std-only middleware-chain abstraction.
//!
//! Layer 2 of the hyper foundation. Generic over `Req` + `Resp` so consuming
//! crates can plug in any concrete request/response type (the hyper-runtime
//! adapter wires `hyper::Request<Body>` / `Response<Body>`).
//!
//! Chain semantics: each `Middleware::handle(&self, req, next)` may either
//!   - call `next.run(req)` to continue down the chain (returning that result
//!     optionally transformed), or
//!   - short-circuit by returning its own `Resp` without calling `next`.
//!
//! This is sync (Resp returned directly). The hyper-runtime adapter (Layer 5)
//! will wrap this in async via a thin spawn-blocking-ish helper, or build a
//! sibling async chain. Keeping the kernel sync means it stays std-only and
//! testable without an async runtime.

/// A chain handler that can be called by middleware to continue down the stack.
pub struct Next<'a, Req, Resp> {
    chain: &'a [Box<dyn Middleware<Req, Resp>>],
    terminal: &'a dyn Fn(Req) -> Resp,
}

impl<Req, Resp> Next<'_, Req, Resp> {
    pub fn run(self, request: Req) -> Resp {
        match self.chain.split_first() {
            None => (self.terminal)(request),
            Some((head, tail)) => head.handle(
                request,
                Next {
                    chain: tail,
                    terminal: self.terminal,
                },
            ),
        }
    }
}

/// Trait every middleware implements. The chain composes these in registered
/// order; the terminal handler runs last when every middleware calls `next.run`.
pub trait Middleware<Req, Resp>: Send + Sync {
    fn handle(&self, request: Req, next: Next<'_, Req, Resp>) -> Resp;
}

/// Composable chain of middlewares + a terminal handler.
pub struct MiddlewareChain<Req, Resp> {
    middlewares: Vec<Box<dyn Middleware<Req, Resp>>>,
}

impl<Req, Resp> Default for MiddlewareChain<Req, Resp> {
    fn default() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }
}

impl<Req, Resp> MiddlewareChain<Req, Resp> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(mut self, middleware: Box<dyn Middleware<Req, Resp>>) -> Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn count(&self) -> usize {
        self.middlewares.len()
    }

    /// Execute the chain against a terminal handler. The terminal handler is
    /// what runs when every middleware has called `next.run(req)`.
    pub fn execute<F>(&self, request: Req, terminal: F) -> Resp
    where
        F: Fn(Req) -> Resp,
    {
        let next = Next {
            chain: &self.middlewares,
            terminal: &terminal,
        };
        next.run(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct Counter(Arc<AtomicUsize>);
    impl<Req, Resp> Middleware<Req, Resp> for Counter
    where
        Req: Send + Sync + 'static,
        Resp: Send + Sync + 'static,
    {
        fn handle(&self, request: Req, next: Next<'_, Req, Resp>) -> Resp {
            self.0.fetch_add(1, Ordering::SeqCst);
            next.run(request)
        }
    }

    struct ShortCircuit<R>(R);
    impl<Req, Resp> Middleware<Req, Resp> for ShortCircuit<Resp>
    where
        Req: Send + Sync + 'static,
        Resp: Clone + Send + Sync + 'static,
    {
        fn handle(&self, _request: Req, _next: Next<'_, Req, Resp>) -> Resp {
            self.0.clone()
        }
    }

    #[test]
    fn empty_chain_runs_terminal() {
        let chain: MiddlewareChain<&'static str, String> = MiddlewareChain::new();
        let response = chain.execute("hello", |req| format!("handled:{req}"));
        assert_eq!(response, "handled:hello");
    }

    #[test]
    fn single_middleware_invokes_next() {
        let counter = Arc::new(AtomicUsize::new(0));
        let chain: MiddlewareChain<&'static str, String> =
            MiddlewareChain::new().push(Box::new(Counter(counter.clone())));
        let response = chain.execute("x", |req| format!("handled:{req}"));
        assert_eq!(response, "handled:x");
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn multiple_middleware_all_invoked() {
        let counter = Arc::new(AtomicUsize::new(0));
        let chain: MiddlewareChain<&'static str, String> = MiddlewareChain::new()
            .push(Box::new(Counter(counter.clone())))
            .push(Box::new(Counter(counter.clone())))
            .push(Box::new(Counter(counter.clone())));
        let _ = chain.execute("x", |_| String::from("done"));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn short_circuit_skips_terminal() {
        let counter = Arc::new(AtomicUsize::new(0));
        let chain: MiddlewareChain<&'static str, String> = MiddlewareChain::new()
            .push(Box::new(Counter(counter.clone())))
            .push(Box::new(ShortCircuit(String::from("denied"))))
            .push(Box::new(Counter(counter.clone())));
        let response = chain.execute("x", |_| String::from("should-not-run"));
        assert_eq!(response, "denied");
        // First counter ran (called next), short-circuit ran, third counter did NOT run.
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn count_reflects_pushed_middleware() {
        let chain: MiddlewareChain<&'static str, String> = MiddlewareChain::new()
            .push(Box::new(Counter(Arc::new(AtomicUsize::new(0)))))
            .push(Box::new(Counter(Arc::new(AtomicUsize::new(0)))));
        assert_eq!(chain.count(), 2);
    }

    #[test]
    fn middleware_runs_in_registered_order() {
        struct Tag {
            tag: &'static str,
            log: Arc<std::sync::Mutex<Vec<&'static str>>>,
        }
        impl Middleware<&'static str, String> for Tag {
            fn handle(&self, req: &'static str, next: Next<'_, &'static str, String>) -> String {
                self.log.lock().unwrap().push(self.tag);
                next.run(req)
            }
        }
        let log = Arc::new(std::sync::Mutex::new(Vec::new()));
        let chain: MiddlewareChain<&'static str, String> = MiddlewareChain::new()
            .push(Box::new(Tag {
                tag: "a",
                log: log.clone(),
            }))
            .push(Box::new(Tag {
                tag: "b",
                log: log.clone(),
            }))
            .push(Box::new(Tag {
                tag: "c",
                log: log.clone(),
            }));
        let _ = chain.execute("x", |_| String::from("end"));
        let recorded = log.lock().unwrap().clone();
        assert_eq!(recorded, vec!["a", "b", "c"]);
    }
}
