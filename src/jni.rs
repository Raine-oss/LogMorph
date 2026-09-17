// JNI Bridge

use crate::cli::TerminalRenderer;
use crate::engine::AggregationEngine;
use crate::parser::LogStreamReader;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jstring, JNI_TRUE};
use jni::JNIEnv;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

// Native Methods

#[no_mangle]
pub extern "system" fn Java_io_github_raine_logmorph_LogMorphBridge_analyzeLog<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    log_path: JString<'local>,
    summary_only: jboolean,
    plugin_filter: JString<'local>,
    no_color: jboolean,
) -> jstring {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let path_str: String = match env.get_string(&log_path) {
            Ok(js) => js.into(),
            Err(e) => return format!("Error: Failed to read log path: {}", e),
        };

        let filter_opt: Option<String> = if !plugin_filter.is_null() {
            match env.get_string(&plugin_filter) {
                Ok(js) => {
                    let s: String = js.into();
                    if s.trim().is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        };

        let is_summary_only = summary_only == JNI_TRUE;
        let is_no_color = no_color == JNI_TRUE;

        let path = Path::new(&path_str);
        if !path.exists() {
            return format!("Error: Log file not found at: {}", path.display());
        }

        let reader = match LogStreamReader::from_path(path) {
            Ok(r) => r,
            Err(e) => return format!("Error: Failed to open log file {}: {}", path.display(), e),
        };

        let mut engine = AggregationEngine::new();
        for event_res in reader {
            match event_res {
                Ok(event) => engine.feed_event(event),
                Err(e) => return format!("Error: Stream reading error: {}", e),
            }
        }

        let renderer = TerminalRenderer::new(is_no_color);
        renderer.format_all(&engine, is_summary_only, filter_opt.as_deref())
    }));

    let output_str = match result {
        Ok(s) => s,
        Err(_) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                "LogMorph native panic occurred during log analysis",
            );
            return std::ptr::null_mut();
        }
    };

    match env.new_string(&output_str) {
        Ok(js) => js.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// Export JSON Native

#[no_mangle]
pub extern "system" fn Java_io_github_raine_logmorph_LogMorphBridge_exportJson<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    log_path: JString<'local>,
) -> jstring {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let path_str: String = match env.get_string(&log_path) {
            Ok(js) => js.into(),
            Err(e) => return serde_json::json!({ "error": format!("Failed to read log path: {}", e) }).to_string(),
        };

        let path = Path::new(&path_str);
        if !path.exists() {
            return serde_json::json!({ "error": format!("Log file not found at: {}", path.display()) }).to_string();
        }

        let reader = match LogStreamReader::from_path(path) {
            Ok(r) => r,
            Err(e) => return serde_json::json!({ "error": format!("Failed to open log file {}: {}", path.display(), e) }).to_string(),
        };

        let mut engine = AggregationEngine::new();
        for event_res in reader {
            match event_res {
                Ok(event) => engine.feed_event(event),
                Err(e) => return serde_json::json!({ "error": format!("Stream reading error: {}", e) }).to_string(),
            }
        }

        let report = engine.to_report();
        match serde_json::to_string_pretty(&report) {
            Ok(json) => json,
            Err(e) => serde_json::json!({ "error": format!("Serialization error: {}", e) }).to_string(),
        }
    }));

    let output_str = match result {
        Ok(s) => s,
        Err(_) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                "LogMorph native panic occurred during exportJson",
            );
            return std::ptr::null_mut();
        }
    };

    match env.new_string(&output_str) {
        Ok(js) => js.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}
