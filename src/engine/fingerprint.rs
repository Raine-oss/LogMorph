// Fingerprint Generator

use crate::engine::frame_filter::FrameFilter;
use crate::models::stack_trace::{StackFrame, StackTraceBlock};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;

// Signature Key

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSignatureKey {
    pub hash: u64,
    pub structural_identity: String,
}

impl PartialEq for ErrorSignatureKey {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.structural_identity == other.structural_identity
    }
}

impl Eq for ErrorSignatureKey {}

impl Hash for ErrorSignatureKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

// Normalizer

pub fn normalize_dynamic_noise(input: &str) -> String {
    static UUID_RE: OnceLock<Regex> = OnceLock::new();
    static TIME_RE: OnceLock<Regex> = OnceLock::new();
    static ADDR_RE: OnceLock<Regex> = OnceLock::new();
    static COORD_RE: OnceLock<Regex> = OnceLock::new();

    let uuid_re = UUID_RE.get_or_init(|| {
        Regex::new(r"(?i)\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b").unwrap()
    });
    let time_re = TIME_RE.get_or_init(|| {
        Regex::new(r"\b\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2}(?:\.\d+)?\b|\b\d{2}:\d{2}:\d{2}(?:\.\d+)?\b").unwrap()
    });
    let addr_re = ADDR_RE.get_or_init(|| {
        Regex::new(r"(?i)(?:@0x[0-9a-f]+|@[0-9a-f]{6,16}|\b0x[0-9a-f]{4,16}\b)").unwrap()
    });
    let coord_re = COORD_RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:x|y|z|pitch|yaw)\s*[:=]\s*[-+]?\d*\.?\d+\b|\(\s*[-+]?\d+\.\d+,\s*[-+]?\d+\.\d+,\s*[-+]?\d+\.\d+\s*\)").unwrap()
    });

    let s = uuid_re.replace_all(input, "<UUID>");
    let s = time_re.replace_all(&s, "<TIME>");
    let s = addr_re.replace_all(&s, "<ADDR>");
    let s = coord_re.replace_all(&s, "<COORD>");
    s.into_owned()
}

// Fingerprint Engine

pub struct FingerprintGenerator;

impl FingerprintGenerator {
    pub fn build_structural_identity(
        primary_exception: &str,
        plugin_name: Option<&str>,
        top_frame: Option<&StackFrame>,
        raw_message: Option<&str>,
    ) -> String {
        let norm_msg = raw_message.map(normalize_dynamic_noise).unwrap_or_default();
        let frame_id = match top_frame {
            Some(f) => format!(
                "{}.{}:{}:{}",
                f.class_name,
                f.method_name,
                f.file_name.as_deref().unwrap_or(""),
                f.line_number.unwrap_or(0)
            ),
            None => String::new(),
        };
        format!(
            "{}|{}|{}|{}",
            primary_exception,
            plugin_name.unwrap_or(""),
            frame_id,
            norm_msg
        )
    }

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
        raw_message: Option<&str>,
        trace: &StackTraceBlock,
    ) -> (ErrorSignatureKey, Option<StackFrame>) {
        let top_frame = FrameFilter::find_top_plugin_frame(trace);
        let hash = Self::generate_signature(primary_exception, plugin_name, top_frame.as_ref());
        let structural_identity = Self::build_structural_identity(
            primary_exception,
            plugin_name,
            top_frame.as_ref(),
            raw_message,
        );
        (
            ErrorSignatureKey {
                hash,
                structural_identity,
            },
            top_frame,
        )
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

    #[test]
    fn test_dynamic_noise_normalization() {
        let msg = "Player uuid 12345678-abcd-1234-ef01-123456789abc failed at x=100.5, y=64.0, z=-200.0 with obj 0x7f9a12bc4500 at 2026-09-17 10:00:00";
        let normalized = normalize_dynamic_noise(msg);
        assert!(normalized.contains("<UUID>"));
        assert!(normalized.contains("<COORD>"));
        assert!(normalized.contains("<ADDR>"));
        assert!(normalized.contains("<TIME>"));
    }

    #[test]
    fn test_semantic_numbers_preserved() {
        let msg = "HTTP 500 Internal Server Error connecting to MySQL at 127.0.0.1:3306 on plugin v1.20.4 (code 1045)";
        let normalized = normalize_dynamic_noise(msg);
        assert!(normalized.contains("HTTP 500"));
        assert!(normalized.contains("3306"));
        assert!(normalized.contains("v1.20.4"));
        assert!(normalized.contains("code 1045"));
    }

    #[test]
    fn test_collision_resistant_keys() {
        let key1 = ErrorSignatureKey {
            hash: 42,
            structural_identity: "ErrorA".to_string(),
        };
        let key2 = ErrorSignatureKey {
            hash: 42,
            structural_identity: "ErrorB".to_string(),
        };
        assert_ne!(key1, key2);
    }
}
