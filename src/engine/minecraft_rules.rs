// Minecraft Rules

use regex::Regex;
use std::sync::LazyLock;

// Rule Patterns

static EVENT_EXCEPTION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Could not pass event (?P<event>\w+) to (?P<plugin>[\w\-]+)\s*v(?P<version>[\w\.\-]+)").unwrap()
});

static PLUGIN_TASK_EXCEPTION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Plugin (?P<plugin>[\w\-]+)\s*v(?P<version>[\w\.\-]+) generated an exception when executing task (?P<task>\d+)").unwrap()
});

static PLUGIN_PREFIX_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[(?P<plugin>[\w\-]+)\]\s*(?P<msg>.*)$").unwrap()
});

// Extraction Metadata

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedPluginInfo {
    pub plugin_name: String,
    pub plugin_version: Option<String>,
    pub event_name: Option<String>,
}

// Extraction Engine

pub struct MinecraftRules;

impl MinecraftRules {
    pub fn extract_from_message(message: &str) -> Option<ExtractedPluginInfo> {
        if let Some(caps) = EVENT_EXCEPTION_REGEX.captures(message) {
            return Some(ExtractedPluginInfo {
                plugin_name: caps.name("plugin").unwrap().as_str().to_string(),
                plugin_version: caps.name("version").map(|v| v.as_str().to_string()),
                event_name: caps.name("event").map(|e| e.as_str().to_string()),
            });
        }

        if let Some(caps) = PLUGIN_TASK_EXCEPTION_REGEX.captures(message) {
            return Some(ExtractedPluginInfo {
                plugin_name: caps.name("plugin").unwrap().as_str().to_string(),
                plugin_version: caps.name("version").map(|v| v.as_str().to_string()),
                event_name: caps.name("task").map(|t| format!("Task #{}", t.as_str())),
            });
        }

        if let Some(caps) = PLUGIN_PREFIX_REGEX.captures(message) {
            return Some(ExtractedPluginInfo {
                plugin_name: caps.name("plugin").unwrap().as_str().to_string(),
                plugin_version: None,
                event_name: None,
            });
        }

        None
    }
}
