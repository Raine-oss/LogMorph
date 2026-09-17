// Frame Filter

use crate::models::stack_trace::{StackFrame, StackTraceBlock};

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
}
