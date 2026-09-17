// Line Parser

use crate::models::log_line::{LogLevel, LogLine};
use crate::models::stack_trace::StackFrame;
use regex::Regex;
use std::sync::LazyLock;

// Regex Patterns

static SINGLE_BRACKET_LOG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[(?P<time>[^\]\s]+(?:\s+[^\]\s]+)?)\s+(?P<level>[A-Z]+)\](?:\s*\[(?P<source>[^\]]+)\])?:\s*(?P<msg>.*)$").unwrap()
});

static MULTI_BRACKET_LOG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[(?P<time>[^\]]+)\]\s*\[(?:(?P<thread>[^/\]]+)/)?(?P<level>[^\]]+)\](?:\s*\[(?P<source>[^\]]+)\])?:\s*(?P<msg>.*)$").unwrap()
});

static STACK_FRAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*at\s+(?P<class_method>[\w\.$<>]+)\((?:(?P<file>[^:]+):(?P<line>\d+)|(?P<native>Native Method)|(?P<unknown>[^)]+))\)(?:\s*~?\[.*\])?$").unwrap()
});

static CAUSED_BY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*Caused by:\s+(?P<exception>[\w\.$]+)(?::\s*(?P<msg>.*))?$").unwrap()
});

static EXCEPTION_HEADER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:Exception in thread \S+\s+)?(?P<exception>[a-zA-Z_$][a-zA-Z\d_$]*(?:\.[a-zA-Z_$][a-zA-Z\d_$]*)*(?:Exception|Error|Throwable|EventException))(?::\s*(?P<msg>.*))?$").unwrap()
});

// Parsed Line Type

#[derive(Debug, PartialEq, Eq)]
pub enum ParsedLine {
    Log(LogLine),
    ExceptionHeader {
        exception: String,
        message: Option<String>,
    },
    CausedBy {
        exception: String,
        message: Option<String>,
    },
    Frame(StackFrame),
    MoreFrames(usize),
    Text(String),
}

// Parser Implementation

pub struct LineParser;

impl LineParser {
    pub fn parse(line: &str) -> ParsedLine {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            return ParsedLine::Text(line.to_string());
        }

        if let Some(caps) = CAUSED_BY_REGEX.captures(trimmed) {
            let exception = caps.name("exception").unwrap().as_str().to_string();
            let message = caps.name("msg").map(|m| m.as_str().to_string());
            return ParsedLine::CausedBy { exception, message };
        }

        if let Some(caps) = STACK_FRAME_REGEX.captures(line) {
            let full_cm = caps.name("class_method").unwrap().as_str();
            let (class_name, method_name) = match full_cm.rfind('.') {
                Some(idx) => (full_cm[..idx].to_string(), full_cm[idx + 1..].to_string()),
                None => (String::new(), full_cm.to_string()),
            };

            let is_native = caps.name("native").is_some();
            let file_name = caps.name("file").map(|f| f.as_str().to_string());
            let line_number = caps
                .name("line")
                .and_then(|l| l.as_str().parse::<u32>().ok());

            return ParsedLine::Frame(StackFrame::new(
                class_name,
                method_name,
                file_name,
                line_number,
                is_native,
            ));
        }

        if trimmed.starts_with("...") && trimmed.ends_with("more") {
            let inner = trimmed
                .trim_start_matches('.')
                .trim_end_matches("more")
                .trim();
            let count = inner.parse::<usize>().unwrap_or(0);
            return ParsedLine::MoreFrames(count);
        }

        if let Some(caps) = MULTI_BRACKET_LOG_REGEX.captures(trimmed) {
            let timestamp = caps.name("time").map(|t| t.as_str().to_string());
            let thread = caps.name("thread").map(|th| th.as_str().to_string());
            let raw_level = caps.name("level").map(|l| l.as_str()).unwrap_or("UNKNOWN");
            let source = caps.name("source").map(|s| s.as_str().to_string());
            let msg = caps.name("msg").map(|m| m.as_str()).unwrap_or("").to_string();

            return ParsedLine::Log(LogLine::new(
                timestamp,
                LogLevel::from_str(raw_level),
                thread,
                source,
                msg,
            ));
        }

        if let Some(caps) = SINGLE_BRACKET_LOG_REGEX.captures(trimmed) {
            let timestamp = caps.name("time").map(|t| t.as_str().to_string());
            let raw_level = caps.name("level").map(|l| l.as_str()).unwrap_or("UNKNOWN");
            let source = caps.name("source").map(|s| s.as_str().to_string());
            let msg = caps.name("msg").map(|m| m.as_str()).unwrap_or("").to_string();

            return ParsedLine::Log(LogLine::new(
                timestamp,
                LogLevel::from_str(raw_level),
                None,
                source,
                msg,
            ));
        }

        if let Some(caps) = EXCEPTION_HEADER_REGEX.captures(trimmed) {
            let exception = caps.name("exception").unwrap().as_str().to_string();
            let message = caps.name("msg").map(|m| m.as_str().to_string());
            return ParsedLine::ExceptionHeader { exception, message };
        }

        ParsedLine::Text(line.to_string())
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_bracket_log() {
        let line = "[12:34:56 INFO]: Hello world";
        let parsed = LineParser::parse(line);
        match parsed {
            ParsedLine::Log(log) => {
                assert_eq!(log.timestamp.as_deref(), Some("12:34:56"));
                assert_eq!(log.level, LogLevel::Info);
                assert_eq!(log.message, "Hello world");
            }
            _ => panic!("Expected ParsedLine::Log"),
        }
    }

    #[test]
    fn test_parse_multi_bracket_log() {
        let line = "[12:34:56] [Server thread/WARN] [MyPlugin]: Warning message";
        let parsed = LineParser::parse(line);
        match parsed {
            ParsedLine::Log(log) => {
                assert_eq!(log.timestamp.as_deref(), Some("12:34:56"));
                assert_eq!(log.level, LogLevel::Warn);
                assert_eq!(log.thread_name.as_deref(), Some("Server thread"));
                assert_eq!(log.source.as_deref(), Some("MyPlugin"));
                assert_eq!(log.message, "Warning message");
            }
            _ => panic!("Expected ParsedLine::Log"),
        }
    }

    #[test]
    fn test_parse_stack_frame() {
        let line = "\tat com.example.plugin.Main.onEnable(Main.java:42)";
        let parsed = LineParser::parse(line);
        match parsed {
            ParsedLine::Frame(frame) => {
                assert_eq!(frame.class_name, "com.example.plugin.Main");
                assert_eq!(frame.method_name, "onEnable");
                assert_eq!(frame.file_name.as_deref(), Some("Main.java"));
                assert_eq!(frame.line_number, Some(42));
                assert!(!frame.is_native);
            }
            _ => panic!("Expected ParsedLine::Frame"),
        }
    }

    #[test]
    fn test_parse_native_stack_frame() {
        let line = "\tat java.lang.ClassLoader.loadLibrary(Native Method)";
        let parsed = LineParser::parse(line);
        match parsed {
            ParsedLine::Frame(frame) => {
                assert_eq!(frame.class_name, "java.lang.ClassLoader");
                assert_eq!(frame.method_name, "loadLibrary");
                assert!(frame.is_native);
            }
            _ => panic!("Expected ParsedLine::Frame"),
        }
    }

    #[test]
    fn test_parse_caused_by() {
        let line = "Caused by: java.io.IOException: Permission denied";
        let parsed = LineParser::parse(line);
        match parsed {
            ParsedLine::CausedBy { exception, message } => {
                assert_eq!(exception, "java.io.IOException");
                assert_eq!(message.as_deref(), Some("Permission denied"));
            }
            _ => panic!("Expected ParsedLine::CausedBy"),
        }
    }

    #[test]
    fn test_parse_more_frames() {
        let line = "\t... 14 more";
        let parsed = LineParser::parse(line);
        assert_eq!(parsed, ParsedLine::MoreFrames(14));
    }
}
