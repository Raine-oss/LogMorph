// Stack Frame

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StackFrame {
    pub class_name: String,
    pub method_name: String,
    pub file_name: Option<String>,
    pub line_number: Option<u32>,
    pub is_native: bool,
}

impl StackFrame {
    pub fn new(
        class_name: String,
        method_name: String,
        file_name: Option<String>,
        line_number: Option<u32>,
        is_native: bool,
    ) -> Self {
        Self {
            class_name,
            method_name,
            file_name,
            line_number,
            is_native,
        }
    }
}

// Stack Trace Block

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackTraceBlock {
    pub primary_exception: String,
    pub exception_message: Option<String>,
    pub frames: Vec<StackFrame>,
    pub caused_by: Option<Box<StackTraceBlock>>,
}

impl StackTraceBlock {
    pub fn new(
        primary_exception: String,
        exception_message: Option<String>,
        frames: Vec<StackFrame>,
        caused_by: Option<Box<StackTraceBlock>>,
    ) -> Self {
        Self {
            primary_exception,
            exception_message,
            frames,
            caused_by,
        }
    }
}
