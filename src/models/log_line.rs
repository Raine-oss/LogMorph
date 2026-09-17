// Log Level

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
    Trace,
    Unknown,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "INFO" => LogLevel::Info,
            "WARN" | "WARNING" => LogLevel::Warn,
            "ERROR" | "SEVERE" | "FATAL" => LogLevel::Error,
            "DEBUG" => LogLevel::Debug,
            "TRACE" => LogLevel::Trace,
            _ => LogLevel::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
            LogLevel::Unknown => "UNKNOWN",
        }
    }
}

// Log Line

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LogLine {
    pub timestamp: Option<String>,
    pub level: LogLevel,
    pub thread_name: Option<String>,
    pub source: Option<String>,
    pub message: String,
}

impl LogLine {
    pub fn new(
        timestamp: Option<String>,
        level: LogLevel,
        thread_name: Option<String>,
        source: Option<String>,
        message: String,
    ) -> Self {
        Self {
            timestamp,
            level,
            thread_name,
            source,
            message,
        }
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from_str("INFO"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("warn"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("SEVERE"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("FATAL"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("debug"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("trace"), LogLevel::Trace);
        assert_eq!(LogLevel::from_str("anything_else"), LogLevel::Unknown);
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }
}
