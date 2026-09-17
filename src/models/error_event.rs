// Attribution

use crate::models::stack_trace::{StackFrame, StackTraceBlock};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AttributionKind {
    Confirmed,
    Inferred,
    Unknown,
}

impl AttributionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AttributionKind::Confirmed => "Confirmed",
            AttributionKind::Inferred => "Inferred from stack frame",
            AttributionKind::Unknown => "Unknown",
        }
    }
}

// Error Event

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AggregatedError {
    pub signature_hash: u64,
    pub primary_exception: String,
    pub exception_message: Option<String>,
    pub plugin_name: Option<String>,
    pub event_name: Option<String>,
    pub attribution: AttributionKind,
    pub top_plugin_frame: Option<StackFrame>,
    pub occurrences: usize,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
    pub sample_trace: StackTraceBlock,
}

impl AggregatedError {
    pub fn new(
        signature_hash: u64,
        primary_exception: String,
        raw_message: Option<String>,
        plugin_name: Option<String>,
        event_name: Option<String>,
        attribution: AttributionKind,
        top_plugin_frame: Option<StackFrame>,
        timestamp: Option<String>,
        sample_trace: StackTraceBlock,
    ) -> Self {
        let exception_message = raw_message.and_then(|m| {
            let trimmed = m.trim();
            if trimmed.is_empty() || trimmed == "null" {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        Self {
            signature_hash,
            primary_exception,
            exception_message,
            plugin_name,
            event_name,
            attribution,
            top_plugin_frame,
            occurrences: 1,
            first_seen: timestamp.clone(),
            last_seen: timestamp,
            sample_trace,
        }
    }

    pub fn record_occurrence(&mut self, timestamp: Option<String>) {
        self.occurrences += 1;
        if timestamp.is_some() {
            self.last_seen = timestamp;
        }
    }
}
