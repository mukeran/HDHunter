use std::borrow::Cow;

use crate::observers::HttpParamObserver;
use libafl::events::EventFirer;
use libafl::executors::ExitKind;
use libafl::feedbacks::Feedback;
use libafl::inputs::Input;
use libafl::observers::ObserversTuple;
use libafl::state::State;
use libafl_bolts::tuples::{Handle, Handled, MatchNameRef};
use libafl_bolts::Named;
use log::debug;

pub struct HttpParamFeedback {
    name: Cow<'static, str>,
    observers: (Handle<HttpParamObserver>, Handle<HttpParamObserver>),
}

impl HttpParamFeedback {
    pub fn new<S>(name: S, observers: &(HttpParamObserver, HttpParamObserver)) -> Self
    where
        S: Into<String>,
    {
        HttpParamFeedback {
            name: Cow::from(name.into()),
            observers: (observers.0.handle(), observers.1.handle()),
        }
    }
}

impl<I, S> Feedback<S> for HttpParamFeedback
where
    I: Input,
    S: State<Input = I>,
{
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        input: &I,
        observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, libafl::Error>
    where
        EM: EventFirer<State = S>,
        OT: ObserversTuple<S>,
    {
        let o1: &HttpParamObserver = observers
            .get(&self.observers.0)
            .ok_or_else(|| {
                libafl::Error::illegal_argument(format!(
                    "HttpParamFeedback: Observer {} not found",
                    self.observers.0.name()
                ))
            })
            .unwrap();
        let o2: &HttpParamObserver = observers
            .get(&self.observers.1)
            .ok_or_else(|| {
                libafl::Error::illegal_argument(format!(
                    "HttpParamFeedback: Observer {} not found",
                    self.observers.1.name()
                ))
            })
            .unwrap();
        let p1 = o1.http_param();
        let p2 = o2.http_param();

        macro_rules! rule {
            ($cond:expr) => {
                if $cond {
                    debug!("Found interesting input {}, reason: {}", input.generate_name(0), stringify!($cond));
                    debug!("First: {:?}", p1);
                    debug!("Second: {:?}", p2);
                    return Ok(true);
                }
            };
        }

        macro_rules! is_error {
            ($status:expr) => {
                $status == 0 || ($status >= 400 && $status < 600)
            };
        }

        rule!(p1.message_count != p2.message_count);
        rule!(p1.message_processed != p2.message_processed);

        for i in 0..10 {
            rule!(p1.body_length[i] != p2.body_length[i]);
            if is_error!(p1.status[i]) && is_error!(p2.status[i]) {
                continue;
            }
            rule!(p1.chunked_encoding[i] != p2.chunked_encoding[i]);
            rule!(p1.content_length[i] != p2.content_length[i]);
            rule!(p1.consumed_length[i] != p2.consumed_length[i]);
        }

        Ok(false)
    }
}

impl Named for HttpParamFeedback {
    fn name(&self) -> &Cow<'static, str> {
        &self.name
    }
}

pub struct HttpParamDebugFeedback {
    name: Cow<'static, str>,
    observer_handle: Handle<HttpParamObserver>,
}

impl HttpParamDebugFeedback {
    pub fn new<S>(name: S, observer: &HttpParamObserver) -> Self
    where
        S: Into<String>,
    {
        HttpParamDebugFeedback {
            name: Cow::from(name.into()),
            observer_handle: observer.handle()
        }
    }
}

impl<I, S> Feedback<S> for HttpParamDebugFeedback
where
    I: Input,
    S: State<Input = I>,
{
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _input: &I,
        observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, libafl::Error>
    where
        EM: EventFirer<State = S>,
        OT: ObserversTuple<S>,
    {
        let o: &HttpParamObserver = observers
            .get(&self.observer_handle)
            .ok_or_else(|| {
                libafl::Error::illegal_argument(format!(
                    "HttpParamDebugFeedback: Observer {} not found",
                    self.observer_handle.name()
                ))
            })
            .unwrap();
        let p = o.http_param();
        debug!("HttpParam: {:?}", p);

        Ok(false)
    }
}

impl Named for HttpParamDebugFeedback {
    fn name(&self) -> &Cow<'static, str> {
        &self.name
    }
}