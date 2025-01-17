use std::borrow::Cow;

use libafl::inputs::UsesInput;
use libafl::observers::Observer;
use libafl_bolts::ownedref::OwnedMutPtr;
use libafl_bolts::{Error, Named};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct HttpParam {
    pub content_length: [i64; 10],
    pub chunked_encoding: [i8; 10],
    pub consumed_length: [i64; 10],
    pub body_length: [i64; 10],
    pub message_count: i32,
    pub message_processed: i8,
    pub status: [i16; 10],
    pub order: [i32; 10],
}

impl HttpParam {
    pub fn clear(&mut self) {
        self.content_length = [-1; 10];
        self.chunked_encoding = [0; 10];
        self.consumed_length = [-1; 10];
        self.body_length = [-1; 10];
        self.message_count = 0;
        self.message_processed = 0;
        self.status = [0; 10];
        self.order = [0; 10];
    }
}

#[derive(Serialize, Deserialize)]
pub struct HttpParamObserver {
    pub name: Cow<'static, str>,
    pub http_param_ptr: OwnedMutPtr<u8>,
}

impl HttpParamObserver {
    pub fn new<S>(name: S, raw_http_param: *mut u8) -> HttpParamObserver
    where
        S: Into<String>,
    {
        HttpParamObserver {
            name: Cow::from(name.into()),
            http_param_ptr: OwnedMutPtr::Ptr(raw_http_param),
        }
    }

    pub fn http_param(&self) -> &HttpParam {
        unsafe { &*((self.http_param_ptr.as_ref() as *const u8) as *const HttpParam) }
    }

    pub fn http_param_mut(&mut self) -> &mut HttpParam {
        unsafe { &mut *((self.http_param_ptr.as_mut() as *mut u8) as *mut HttpParam) }
    }
}

impl<S> Observer<S> for HttpParamObserver
where
    S: UsesInput,
{
    fn pre_exec(&mut self, _state: &mut S, _input: &S::Input) -> Result<(), Error> {
        self.http_param_mut().clear();
        Ok(())
    }
}

impl Named for HttpParamObserver {
    fn name(&self) -> &Cow<'static, str> {
        &self.name
    }
}
