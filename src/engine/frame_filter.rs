// Frame Filter

use crate::models::stack_trace::{StackFrame, StackTraceBlock};
use std::collections::HashSet;

// Frame Classification

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    Framework,
    Plugin,
    Unknown,
}

// Framework Prefixes

static FRAMEWORK_PREFIXES: &[&str] = &[
    "java.",
    "javax.",
    "jdk.",
    "sun.",
    "net.minecraft.",
    "org.bukkit.",
    "com.destroystokyo.paper.",
    "io.papermc.paper.",
    "org.spigotmc.",
    "co.aikar.timings.",
    "net.md_5.bungee.",
    "io.netty.",
    "com.google.",
    "org.apache.",
    "org.slf4j.",
    "it.unimi.dsi.fastutil.",
    "org.yaml.snakeyaml.",
    "com.mojang.",
    "com.zaxxer.hikari.",
    "com.mysql.",
    "org.sqlite.",
    "org.mariadb.",
    "org.checkerframework.",
    "org.jetbrains.",
    "com.mongodb.",
    "redis.clients.",
];

// Frame Classifier

pub struct FrameFilter;

impl FrameFilter {
    pub fn is_framework_frame(frame: &StackFrame) -> bool {
        let class = &frame.class_name;
        FRAMEWORK_PREFIXES.iter().any(|prefix| class.starts_with(prefix))
    }

    pub fn classify_frame(frame: &StackFrame, known_plugins: &HashSet<String>) -> FrameKind {
        if Self::is_framework_frame(frame) {
            return FrameKind::Framework;
        }

        let lower_class = frame.class_name.to_ascii_lowercase();
        let matches_known = known_plugins.iter().any(|plugin| {
            let lower_plugin = plugin.to_ascii_lowercase();
            if lower_plugin.is_empty() {
                return false;
            }
            if lower_class.contains(&lower_plugin) {
                return true;
            }

            let plugin_tokens: Vec<&str> = lower_plugin
                .split(|c: char| !c.is_alphanumeric())
                .filter(|s| s.len() >= 3)
                .collect();
            for token in &plugin_tokens {
                if lower_class.contains(token) {
                    return true;
                }
            }

            let class_tokens: Vec<&str> = lower_class
                .split('.')
                .filter(|s| s.len() >= 3)
                .collect();
            for token in &class_tokens {
                if lower_plugin.contains(token) {
                    return true;
                }
            }

            false
        });

        if matches_known || lower_class.contains("plugin") {
            FrameKind::Plugin
        } else {
            FrameKind::Unknown
        }
    }

    pub fn is_plugin_frame(frame: &StackFrame, known_plugins: &HashSet<String>) -> bool {
        Self::classify_frame(frame, known_plugins) == FrameKind::Plugin
    }

    pub fn find_top_plugin_frame(
        trace: &StackTraceBlock,
        known_plugins: &HashSet<String>,
    ) -> Option<StackFrame> {
        for frame in &trace.frames {
            if Self::is_plugin_frame(frame, known_plugins) {
                return Some(frame.clone());
            }
        }

        if let Some(ref caused_by) = trace.caused_by {
            if let Some(frame) = Self::find_top_plugin_frame(caused_by, known_plugins) {
                return Some(frame);
            }
        }

        None
    }

    pub fn collect_plugin_namespaces(
        trace: &StackTraceBlock,
        known_plugins: &HashSet<String>,
    ) -> Vec<String> {
        let mut frames = Vec::new();
        Self::collect_plugin_frames(trace, known_plugins, &mut frames);

        let mut namespaces = HashSet::new();
        for frame in frames {
            let parts: Vec<&str> = frame.class_name.split('.').collect();
            if parts.len() >= 3 {
                let ns = parts[..3].join(".");
                namespaces.insert(ns);
            } else if parts.len() >= 2 {
                let ns = parts[..2].join(".");
                namespaces.insert(ns);
            }
        }

        namespaces.into_iter().collect()
    }

    fn collect_plugin_frames(
        trace: &StackTraceBlock,
        known_plugins: &HashSet<String>,
        out: &mut Vec<StackFrame>,
    ) {
        for frame in &trace.frames {
            if Self::is_plugin_frame(frame, known_plugins) {
                out.push(frame.clone());
            }
        }
        if let Some(ref caused) = trace.caused_by {
            Self::collect_plugin_frames(caused, known_plugins, out);
        }
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_vs_plugin_frames() {
        let mut known = HashSet::new();
        known.insert("factions".to_string());

        let nms_frame = StackFrame::new(
            "net.minecraft.server.MinecraftServer".to_string(),
            "tick".to_string(),
            None,
            Some(100),
            false,
        );
        assert!(FrameFilter::is_framework_frame(&nms_frame));
        assert!(!FrameFilter::is_plugin_frame(&nms_frame, &known));
        assert_eq!(FrameFilter::classify_frame(&nms_frame, &known), FrameKind::Framework);

        let plugin_frame = StackFrame::new(
            "com.massivecraft.factions.Board".to_string(),
            "getIdAt".to_string(),
            Some("Board.java".to_string()),
            Some(54),
            false,
        );
        assert!(!FrameFilter::is_framework_frame(&plugin_frame));
        assert!(FrameFilter::is_plugin_frame(&plugin_frame, &known));
        assert_eq!(FrameFilter::classify_frame(&plugin_frame, &known), FrameKind::Plugin);

        let unknown_frame = StackFrame::new(
            "org.thirdparty.lib.Helper".to_string(),
            "doWork".to_string(),
            None,
            Some(10),
            false,
        );
        assert!(!FrameFilter::is_framework_frame(&unknown_frame));
        assert!(!FrameFilter::is_plugin_frame(&unknown_frame, &known));
        assert_eq!(FrameFilter::classify_frame(&unknown_frame, &known), FrameKind::Unknown);
    }

    #[test]
    fn test_collect_plugin_namespaces() {
        let mut known = HashSet::new();
        known.insert("pluginA".to_string());
        known.insert("pluginB".to_string());

        let frame1 = StackFrame::new(
            "com.pluginA.listener.MyListener".to_string(),
            "onEvent".to_string(),
            None,
            Some(10),
            false,
        );
        let frame2 = StackFrame::new(
            "com.pluginB.manager.Manager".to_string(),
            "execute".to_string(),
            None,
            Some(20),
            false,
        );
        let block = StackTraceBlock::new(
            "java.lang.Exception".to_string(),
            None,
            vec![frame1, frame2],
            None,
        );
        let namespaces = FrameFilter::collect_plugin_namespaces(&block, &known);
        assert_eq!(namespaces.len(), 2);
    }
}
