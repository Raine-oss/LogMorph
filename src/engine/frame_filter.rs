// Frame Filter

use crate::models::stack_trace::{StackFrame, StackTraceBlock};
use std::collections::HashSet;

// Framework Prefixes

static FRAMEWORK_PREFIXES: &[&str] = &[
    "net.minecraft.",
    "org.bukkit.craftbukkit.",
    "org.bukkit.plugin.java.",
    "org.bukkit.plugin.SimplePluginManager",
    "org.bukkit.plugin.RegisteredListener",
    "org.bukkit.plugin.TimedRegisteredListener",
    "com.destroystokyo.paper.",
    "io.papermc.paper.",
    "java.lang.reflect.",
    "java.base/java.lang.reflect.",
    "jdk.internal.reflect.",
    "sun.reflect.",
    "java.lang.Thread.",
    "java.base/java.lang.Thread.",
    "java.util.concurrent.",
    "java.base/java.util.concurrent.",
    "co.aikar.timings.",
];

// Frame Classifier

pub struct FrameFilter;

impl FrameFilter {
    pub fn is_framework_frame(frame: &StackFrame) -> bool {
        let class = &frame.class_name;
        FRAMEWORK_PREFIXES.iter().any(|prefix| class.starts_with(prefix))
    }

    pub fn is_plugin_frame(frame: &StackFrame) -> bool {
        !Self::is_framework_frame(frame)
    }

    pub fn find_top_plugin_frame(trace: &StackTraceBlock) -> Option<StackFrame> {
        for frame in &trace.frames {
            if Self::is_plugin_frame(frame) {
                return Some(frame.clone());
            }
        }

        if let Some(ref caused_by) = trace.caused_by {
            if let Some(frame) = Self::find_top_plugin_frame(caused_by) {
                return Some(frame);
            }
        }

        trace.frames.first().cloned()
    }

    pub fn collect_plugin_namespaces(trace: &StackTraceBlock) -> Vec<String> {
        let mut frames = Vec::new();
        Self::collect_plugin_frames(trace, &mut frames);

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

    fn collect_plugin_frames(trace: &StackTraceBlock, out: &mut Vec<StackFrame>) {
        for frame in &trace.frames {
            if Self::is_plugin_frame(frame) {
                out.push(frame.clone());
            }
        }
        if let Some(ref caused) = trace.caused_by {
            Self::collect_plugin_frames(caused, out);
        }
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_vs_plugin_frames() {
        let nms_frame = StackFrame::new(
            "net.minecraft.server.MinecraftServer".to_string(),
            "tick".to_string(),
            None,
            Some(100),
            false,
        );
        assert!(FrameFilter::is_framework_frame(&nms_frame));
        assert!(!FrameFilter::is_plugin_frame(&nms_frame));

        let plugin_frame = StackFrame::new(
            "com.massivecraft.factions.Board".to_string(),
            "getIdAt".to_string(),
            Some("Board.java".to_string()),
            Some(54),
            false,
        );
        assert!(!FrameFilter::is_framework_frame(&plugin_frame));
        assert!(FrameFilter::is_plugin_frame(&plugin_frame));
    }

    #[test]
    fn test_collect_plugin_namespaces() {
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
        let namespaces = FrameFilter::collect_plugin_namespaces(&block);
        assert_eq!(namespaces.len(), 2);
    }
}
