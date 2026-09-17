// Integration Tests

use logmorph::engine::aggregation::AnalysisReport;
use logmorph::engine::fingerprint::normalize_dynamic_noise;
use logmorph::engine::{AggregationEngine, FrameFilter, MinecraftRules};
use logmorph::models::error_event::AttributionKind;
use logmorph::models::stack_trace::{StackFrame, StackTraceBlock};
use logmorph::parser::{LogStreamReader, ParsedEvent};
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
    assert_eq!(err.attribution, AttributionKind::Confirmed);

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

#[test]
fn test_dynamic_normalization_with_semantic_numbers() {
    let msg = "Failed to connect to 127.0.0.1:25565 via HTTP 500 on v1.20.4 for player aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee at x=10.5, y=64.0, z=-50.0 (error 1045)";
    let normalized = normalize_dynamic_noise(msg);

    assert!(normalized.contains("HTTP 500"));
    assert!(normalized.contains("25565"));
    assert!(normalized.contains("v1.20.4"));
    assert!(normalized.contains("1045"));
    assert!(normalized.contains("<UUID>"));
    assert!(normalized.contains("<COORD>"));
}

#[test]
fn test_max_signatures_order_retention() {
    let mut engine = AggregationEngine::with_max_signatures(Some(2));

    let trace1 = StackTraceBlock::new(
        "com.plugin.Exception1".to_string(),
        Some("Error 1".to_string()),
        vec![],
        None,
    );
    let trace2 = StackTraceBlock::new(
        "com.plugin.Exception2".to_string(),
        Some("Error 2".to_string()),
        vec![],
        None,
    );
    let trace3 = StackTraceBlock::new(
        "com.plugin.Exception3".to_string(),
        Some("Error 3".to_string()),
        vec![],
        None,
    );

    engine.feed_event(ParsedEvent::ErrorWithTrace {
        log: None,
        trace: trace1.clone(),
        line_count: 1,
    });
    engine.feed_event(ParsedEvent::ErrorWithTrace {
        log: None,
        trace: trace2.clone(),
        line_count: 1,
    });
    engine.feed_event(ParsedEvent::ErrorWithTrace {
        log: None,
        trace: trace3.clone(),
        line_count: 1,
    });

    assert_eq!(engine.stats().unique_signatures, 2);
    assert_eq!(engine.stats().dropped_signatures, 1);

    // Existing signature continues to increment occurrences
    engine.feed_event(ParsedEvent::ErrorWithTrace {
        log: None,
        trace: trace1,
        line_count: 1,
    });

    let errors = engine.aggregated_errors();
    assert_eq!(errors.len(), 2);
    let first = errors.iter().find(|e| e.primary_exception == "com.plugin.Exception1").unwrap();
    assert_eq!(first.occurrences, 2);
}

#[test]
fn test_attribution_ambiguous_vs_detected() {
    let frame_a = StackFrame::new(
        "com.pluginA.listener.MyListener".to_string(),
        "onEvent".to_string(),
        None,
        Some(10),
        false,
    );
    let frame_b = StackFrame::new(
        "com.pluginB.manager.Manager".to_string(),
        "execute".to_string(),
        None,
        Some(20),
        false,
    );

    // Ambiguous trace with 2 distinct plugin namespaces
    let trace_ambiguous = StackTraceBlock::new(
        "java.lang.NullPointerException".to_string(),
        None,
        vec![frame_a.clone(), frame_b],
        None,
    );
    let mut engine = AggregationEngine::new();
    engine.feed_event(ParsedEvent::ErrorWithTrace {
        log: None,
        trace: trace_ambiguous,
        line_count: 2,
    });

    let errors = engine.aggregated_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].attribution, AttributionKind::Ambiguous);

    // Single plugin namespace
    let trace_single = StackTraceBlock::new(
        "java.lang.IllegalStateException".to_string(),
        None,
        vec![frame_a],
        None,
    );
    let mut engine2 = AggregationEngine::new();
    engine2.feed_event(ParsedEvent::ErrorWithTrace {
        log: None,
        trace: trace_single,
        line_count: 1,
    });
    let errors2 = engine2.aggregated_errors();
    assert_eq!(errors2.len(), 1);
    assert_eq!(errors2[0].attribution, AttributionKind::DetectedFromStackFrame);
}

#[test]
fn test_json_schema_roundtrip() {
    let path = fixture_path("paper_sample.log");
    let reader = LogStreamReader::from_path(&path).unwrap();
    let mut engine = AggregationEngine::new();
    for event in reader {
        engine.feed_event(event.unwrap());
    }

    let report = engine.to_report();
    let json_str = serde_json::to_string_pretty(&report).expect("Serialization failed");

    // Ensure snake_case
    assert!(json_str.contains("\"attribution\": \"confirmed\""));
    assert!(json_str.contains("\"total_lines\""));
    assert!(json_str.contains("\"primary_exception\""));

    // Ensure roundtrip deserialization works
    let deserialized: AnalysisReport = serde_json::from_str(&json_str).expect("Deserialization failed");
    assert_eq!(deserialized.stats.total_exceptions, 1);
    assert_eq!(deserialized.aggregated_errors.len(), 1);
    assert_eq!(deserialized.aggregated_errors[0].attribution, AttributionKind::Confirmed);
}

#[test]
fn test_compressed_log_gz_reading() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::fs::File;
    use std::io::Write;

    let sample_text = "\
[12:00:00 INFO]: Server starting
[12:00:01 ERROR]: Something failed
java.lang.NullPointerException: Null entity
\tat com.example.plugin.Main.tick(Main.java:30)
";
    let gz_path = fixture_path("test_sample.log.gz");
    {
        let file = File::create(&gz_path).expect("Failed to create gz file");
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(sample_text.as_bytes()).expect("Write failed");
        encoder.finish().expect("Finish failed");
    }

    let reader = LogStreamReader::from_path(&gz_path).expect("Should open gz file");
    let mut engine = AggregationEngine::new();
    for event in reader {
        engine.feed_event(event.unwrap());
    }

    let _ = std::fs::remove_file(&gz_path);

    assert_eq!(engine.stats().total_exceptions, 1);
    assert_eq!(engine.stats().unique_signatures, 1);
}
