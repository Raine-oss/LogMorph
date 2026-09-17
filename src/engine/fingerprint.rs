// Fingerprint Generator

use crate::engine::frame_filter::FrameFilter;
use crate::models::stack_trace::{StackFrame, StackTraceBlock};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Fingerprint Engine

pub struct FingerprintGenerator;

impl FingerprintGenerator {
    pub fn generate_signature(
        primary_exception: &str,
        plugin_name: Option<&str>,
        top_frame: Option<&StackFrame>,
    ) -> u64 {
        let mut hasher = DefaultHasher::new();

        primary_exception.hash(&mut hasher);

        if let Some(plugin) = plugin_name {
            plugin.hash(&mut hasher);
        }

        if let Some(frame) = top_frame {
            frame.class_name.hash(&mut hasher);
            frame.method_name.hash(&mut hasher);
            if let Some(ref file) = frame.file_name {
                file.hash(&mut hasher);
            }
            if let Some(line) = frame.line_number {
                line.hash(&mut hasher);
            }
        }

        hasher.finish()
    }

    pub fn compute_for_trace(
        primary_exception: &str,
        plugin_name: Option<&str>,
        trace: &StackTraceBlock,
    ) -> (u64, Option<StackFrame>) {
        let top_frame = FrameFilter::find_top_plugin_frame(trace);
        let hash = Self::generate_signature(
            primary_exception,
            plugin_name,
            top_frame.as_ref(),
        );
        (hash, top_frame)
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_deterministic() {
        let frame = StackFrame::new(
            "com.example.Test".to_string(),
            "doSomething".to_string(),
            Some("Test.java".to_string()),
            Some(10),
            false,
        );

        let hash1 = FingerprintGenerator::generate_signature(
            "java.lang.NullPointerException",
            Some("PluginA"),
            Some(&frame),
        );

        let hash2 = FingerprintGenerator::generate_signature(
            "java.lang.NullPointerException",
            Some("PluginA"),
            Some(&frame),
        );

        assert_eq!(hash1, hash2);

        let hash3 = FingerprintGenerator::generate_signature(
            "java.lang.IllegalArgumentException",
            Some("PluginA"),
            Some(&frame),
        );

        assert_ne!(hash1, hash3);
    }
}
