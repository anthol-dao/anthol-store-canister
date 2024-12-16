use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct LogEntry {
    timestamp: u64,
    level: LogLevel,
    caller: Option<Principal>,
    message: String,
    context: Option<String>,
}

impl LogEntry {
    pub fn new(
        level: LogLevel,
        caller: Option<Principal>,
        message: &str,
        context: Option<&str>,
    ) -> Self {
        Self {
            timestamp: ic_cdk::api::time(),
            level,
            caller,
            message: message.to_string(),
            context: context.map(|s| s.to_string()),
        }
    }
}

impl Storable for LogEntry {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum LogLevel {
    Debug,
    Trace,
    Info,
    Warn,
    Error,
}
