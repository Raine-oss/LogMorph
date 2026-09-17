// Integration Tests

use logmorph::engine::{AggregationEngine, FrameFilter, MinecraftRules};
use logmorph::parser::LogStreamReader;
use std::io::Cursor;
use std::path::PathBuf;

// Fixture Paths

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

// Tests

#[test]
fn test_paper_sample_parsing() {
    let path = fixture_path("paper_sample.log");
    let reader = LogStreamReader::from_path(&path).expect("Failed to open paper sample fixture");

    let mut engine = AggregationEngine::new();
    for event_res in reader {
        let event = event_res.expect("Error parsing event");
        engine.feed_event(event);
    }

    let stats = engine.stats();
    assert!(stats.total_lines >= 15);
    assert_eq!(stats.total_exceptions, 1);
    assert_eq!(stats.unique_signatures, 1);

    let errors = engine.aggregated_errors();
    assert_eq!(errors.len(), 1);

    let err = errors[0];
    assert_eq!(err.plugin_name.as_deref(), Some("MyCustomPlugin"));
    assert_eq!(err.event_name.as_deref(), Some("PlayerMoveEvent"));
    assert_eq!(err.primary_exception, "org.bukkit.event.EventException");

    let top_frame = err.top_plugin_frame.as_ref().expect("Expected top plugin frame");
    assert!(top_frame.class_name.starts_with("com.example.myplugin"));
    assert!(!FrameFilter::is_framework_frame(top_frame));

    let caused = err.sample_trace.caused_by.as_ref().expect("Expected Caused by");
    assert_eq!(caused.primary_exception, "java.lang.NullPointerException");
}

#[test]
fn test_repeated_error_deduplication() {
    let path = fixture_path("repeated_error.log");
    let reader = LogStreamReader::from_path(&path).expect("Failed to open repeated error fixture");

    let mut engine = AggregationEngine::new();
    for event in reader {
        engine.feed_event(event.unwrap());
    }

    let stats = engine.stats();
    assert_eq!(stats.total_exceptions, 5);
    assert_eq!(stats.unique_signatures, 1);

    let errors = engine.aggregated_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].occurrences, 5);
    assert_eq!(errors[0].first_seen.as_deref(), Some("10:00:00"));
    assert_eq!(errors[0].last_seen.as_deref(), Some("10:00:04"));
}

#[test]
fn test_standard_java_nested_exceptions() {
    let path = fixture_path("standard_java.log");
    let reader = LogStreamReader::from_path(&path).expect("Failed to open standard java fixture");

    let mut engine = AggregationEngine::new();
    for event in reader {
        engine.feed_event(event.unwrap());
    }

    let stats = engine.stats();
    assert_eq!(stats.total_exceptions, 1);
    assert_eq!(stats.unique_signatures, 1);

    let errors = engine.aggregated_errors();
    let err = errors[0];
    assert_eq!(err.primary_exception, "java.lang.RuntimeException");

    let caused_1 = err.sample_trace.caused_by.as_ref().expect("Expected cause 1");
    assert_eq!(caused_1.primary_exception, "java.io.IOException");

    let caused_2 = caused_1.caused_by.as_ref().expect("Expected cause 2");
    assert_eq!(caused_2.primary_exception, "java.lang.NullPointerException");
}

#[test]
fn test_stream_from_cursor() {
    let raw_log = "\
[12:00:00 INFO]: Line 1
[12:00:01 WARN]: Line 2
[12:00:02 ERROR]: Line 3
java.lang.IllegalStateException: Test exception
\tat com.test.App.main(App.java:10)
[12:00:03 INFO]: Line 4
";

    let cursor = Cursor::new(raw_log);
    let reader = LogStreamReader::new(cursor);
    let mut engine = AggregationEngine::new();

    for event in reader {
        engine.feed_event(event.unwrap());
    }

    let stats = engine.stats();
    assert_eq!(stats.info_count, 2);
    assert_eq!(stats.warn_count, 1);
    assert_eq!(stats.error_count, 1);
    assert_eq!(stats.total_exceptions, 1);
    assert_eq!(stats.unique_signatures, 1);
}

#[test]
fn test_minecraft_rules_pattern_extraction() {
    let msg = "Could not pass event BlockBreakEvent to WorldGuard v7.0.9";
    let extracted = MinecraftRules::extract_from_message(msg).expect("Should extract");
    assert_eq!(extracted.plugin_name, "WorldGuard");
    assert_eq!(extracted.plugin_version.as_deref(), Some("7.0.9"));
    assert_eq!(extracted.event_name.as_deref(), Some("BlockBreakEvent"));
}
