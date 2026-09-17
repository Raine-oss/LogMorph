// Error Event

use crate::models::stack_trace::{StackFrame, StackTraceBlock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregatedError {
    pub signature_hash: u64,
    pub primary_exception: String,
    pub exception_message: Option<String>,
    pub plugin_name: Option<String>,
    pub event_name: Option<String>,
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
        exception_message: Option<String>,
        plugin_name: Option<String>,
        event_name: Option<String>,
        top_plugin_frame: Option<StackFrame>,
        timestamp: Option<String>,
        sample_trace: StackTraceBlock,
    ) -> Self {
        Self {
            signature_hash,
            primary_exception,
            exception_message,
            plugin_name,
            event_name,
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
