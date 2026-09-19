//! The method-call loop of one JMAP request. Each call runs on a blocking
//! worker with its own fixed budget; the loop itself stays on the runtime so
//! a `Busy` re-attempt never holds a worker while it waits.
use super::{Jmap, copy, limits, method::method, references, retry, session};
use axum::{
    Json,
    body::Bytes,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use mail_service::Budget;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};

pub(super) struct Run {
    using: Vec<Value>,
    calls: Vec<Value>,
    responses: Vec<Value>,
    remaining: usize,
    created_ids: BTreeMap<String, String>,
}

/// Validate the request envelope (RFC 8620 §3.3); runs on a blocking worker
/// because the body may be as large as the request limit. The error is the
/// `(status, problem type)` pair of the RFC 7807 answer.
pub(super) fn parse(headers: &HeaderMap, body: &Bytes) -> Result<Run, (StatusCode, &'static str)> {
    if !headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(';')
                .next()
                .is_some_and(|v| v.trim().eq_ignore_ascii_case("application/json"))
        })
    {
        return Err((StatusCode::UNSUPPORTED_MEDIA_TYPE, "notJSON"));
    }
    let Ok(mut body) = serde_json::from_slice::<Value>(body) else {
        return Err((StatusCode::BAD_REQUEST, "notJSON"));
    };
    let Some(object) = body.as_object_mut() else {
        return Err((StatusCode::BAD_REQUEST, "notRequest"));
    };
    let (Some(Value::Array(using)), Some(Value::Array(calls))) =
        (object.remove("using"), object.remove("methodCalls"))
    else {
        return Err((StatusCode::BAD_REQUEST, "notRequest"));
    };
    if using.iter().any(|c| !c.is_string()) {
        return Err((StatusCode::BAD_REQUEST, "notRequest"));
    }
    if using.iter().any(|c| !session::known(c)) {
        return Err((StatusCode::BAD_REQUEST, "unknownCapability"));
    }
    if calls.len() > 16 {
        return Err((StatusCode::BAD_REQUEST, "limit"));
    }
    if calls.iter().any(|c| {
        !c.is_array()
            || c.as_array().is_none_or(|a| a.len() != 3)
            || !c[0].is_string()
            || !c[1].is_object()
            || !c[2].is_string()
    }) {
        return Err((StatusCode::BAD_REQUEST, "notRequest"));
    }
    Ok(Run {
        using,
        calls,
        responses: vec![],
        remaining: limits::MAX_RESPONSE,
        created_ids: BTreeMap::new(),
    })
}

impl Run {
    /// Execute call `index` and record its response; `Err(BUSY)` leaves the
    /// run untouched. Returns the implicit `Email/set` a copy asked for.
    fn call(
        &mut self,
        state: &Jmap,
        token: &str,
        index: usize,
        budget: &Budget,
    ) -> Result<Option<Value>, &'static str> {
        let call = &self.calls[index];
        let name = call[0].as_str().unwrap_or_default();
        let capability = session::capability(name);
        let mut destroy = None;
        // Keep room for the largest bounded mutation response before executing
        // a side effect. Read methods receive the remaining materialization budget.
        let result = if self.remaining < 2 * session::MAX_REQUEST {
            Err("limit")
        } else if !self.using.iter().any(|v| v == capability) {
            Err("unknownMethod")
        } else {
            references::resolve(&call[1], &self.responses).and_then(|mut a| {
                if name != "Core/echo" {
                    references::created(&mut a, &self.created_ids);
                }
                let result = method(
                    &state.service,
                    token,
                    name,
                    &a,
                    self.remaining - 2 * session::MAX_REQUEST,
                    &mut destroy,
                    budget,
                );
                if name == "Email/copy"
                    && let Ok(result) = &result
                {
                    destroy = copy::destroy_args(&a, result);
                }
                result
            })
        };
        if result == Err(retry::BUSY) {
            return Err(retry::BUSY);
        }
        self.record(index, name.to_owned(), result);
        Ok(destroy)
    }

    fn record(&mut self, index: usize, name: String, result: Result<Value, &'static str>) {
        let result = result.and_then(|value| {
            if let Err(error) = limits::charge(&value, &mut self.remaining) {
                self.remaining = 0;
                // A committed mutation must retain its success receipt. Its
                // bounded response was reserved before execution.
                if !matches!(
                    name.as_str(),
                    "Identity/set"
                        | "EmailSubmission/set"
                        | "VacationResponse/set"
                        | "Email/set"
                        | "Mailbox/set"
                        | "Email/copy"
                        | "Email/import"
                        | "Blob/copy"
                ) {
                    return Err(error);
                }
            }
            Ok(value)
        });
        if let Ok(value) = &result
            && let Some(objects) = value["created"].as_object()
        {
            for (key, value) in objects {
                if let Some(id) = value["id"].as_str() {
                    self.created_ids.insert(key.clone(), id.to_owned());
                }
            }
        }
        let id = &self.calls[index][2];
        self.responses.push(match result {
            Ok(value) => json!([name, value, id]),
            Err(kind) => json!(["error",{"type":kind},id]),
        });
    }

    /// The implicit `Email/set` after a successful `Email/copy`.
    fn destroy(
        &mut self,
        state: &Jmap,
        token: &str,
        index: usize,
        args: &Value,
        budget: &Budget,
    ) -> Result<(), &'static str> {
        let result = method(
            &state.service,
            token,
            "Email/set",
            args,
            self.remaining,
            &mut None,
            budget,
        );
        if result == Err(retry::BUSY) {
            return Err(retry::BUSY);
        }
        let id = &self.calls[index][2];
        self.responses.push(match result {
            Ok(value) => {
                if limits::charge(&value, &mut self.remaining).is_err() {
                    self.remaining = 0;
                }
                json!(["Email/set", value, id])
            }
            Err(kind) => json!(["error", {"type":kind}, id]),
        });
        Ok(())
    }

    /// The store stayed busy for the whole budget.
    fn fail(&mut self, index: usize) {
        self.responses
            .push(json!(["error",{"type":"serverFail"},self.calls[index][2]]));
    }
}

/// Run every call of a parsed request; the budget of a busy call is spent
/// on the runtime, not on a worker.
pub(super) async fn run(
    state: Jmap,
    token: Arc<str>,
    mut run: Run,
    cancelled: Arc<AtomicBool>,
) -> Result<Response, StatusCode> {
    for index in 0..run.calls.len() {
        let step = {
            let token = token.clone();
            move |state: &Jmap, run: &mut Run, budget: &Budget| {
                run.call(state, &token, index, budget)
            }
        };
        let (next, outcome) = retry::run(&state, &cancelled, run, step).await?;
        run = next;
        let args = match outcome {
            Some(Ok(Some(args))) => args,
            Some(Ok(None)) => continue,
            _ => {
                run.fail(index);
                continue;
            }
        };
        let step = {
            let token = token.clone();
            move |state: &Jmap, run: &mut Run, budget: &Budget| {
                run.destroy(state, &token, index, &args, budget)
            }
        };
        let (next, outcome) = retry::run(&state, &cancelled, run, step).await?;
        run = next;
        if outcome.is_none() {
            run.fail(index);
        }
    }
    Ok(Json(json!({"methodResponses":run.responses,"sessionState":"1"})).into_response())
}
