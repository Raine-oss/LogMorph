// Aggregation Engine

use crate::engine::fingerprint::FingerprintGenerator;
use crate::engine::minecraft_rules::MinecraftRules;
use crate::models::error_event::{AggregatedError, AttributionKind};
use crate::models::log_line::{LogLevel, LogLine};
use crate::models::stack_trace::StackTraceBlock;
use crate::parser::state_machine::ParsedEvent;
use serde::Serialize;
use std::collections::HashMap;

// Aggregation Stats

#[derive(Debug, Default, Clone, Serialize)]
pub struct AggregationStats {
    pub total_lines: usize,
    pub total_log_messages: usize,
    pub info_count: usize,
    pub warn_count: usize,
    pub error_count: usize,
    pub debug_count: usize,
    pub total_exceptions: usize,
    pub unique_signatures: usize,
    pub dropped_signatures: usize,
}

// Plugin Count

#[derive(Debug, Clone, Serialize)]
pub struct PluginCount {
    pub plugin: String,
    pub count: usize,
}

// Analysis Report

#[derive(Debug, Serialize)]
pub struct AnalysisReport<'a> {
    pub stats: &'a AggregationStats,
    pub top_plugins: Vec<PluginCount>,
    pub aggregated_errors: Vec<&'a AggregatedError>,
}

// Engine Implementation

pub struct AggregationEngine {
    errors_by_signature: HashMap<u64, AggregatedError>,
    plugin_error_counts: HashMap<String, usize>,
    stats: AggregationStats,
    max_signatures: Option<usize>,
}

impl AggregationEngine {
    pub fn new() -> Self {
        Self {
            errors_by_signature: HashMap::new(),
            plugin_error_counts: HashMap::new(),
            stats: AggregationStats::default(),
            max_signatures: None,
        }
    }

    pub fn with_max_signatures(max_signatures: Option<usize>) -> Self {
        Self {
            errors_by_signature: HashMap::new(),
            plugin_error_counts: HashMap::new(),
            stats: AggregationStats::default(),
            max_signatures,
        }
    }

    pub fn feed_event(&mut self, event: ParsedEvent) {
        match event {
            ParsedEvent::Line(log) => {
                self.stats.total_lines += 1;
                self.record_log_line(&log);
            }
            ParsedEvent::ErrorWithTrace {
                log,
                trace,
                line_count,
            } => {
                self.stats.total_lines += line_count;
                if let Some(ref l) = log {
                    self.record_log_line(l);
                }
                self.record_error(log, trace);
            }
            ParsedEvent::Raw(_) => {
                self.stats.total_lines += 1;
            }
        }
    }

    fn record_log_line(&mut self, log: &LogLine) {
        self.stats.total_log_messages += 1;
        match log.level {
            LogLevel::Info => self.stats.info_count += 1,
            LogLevel::Warn => self.stats.warn_count += 1,
            LogLevel::Error => self.stats.error_count += 1,
            LogLevel::Debug => self.stats.debug_count += 1,
            _ => {}
        }
    }

    fn record_error(&mut self, log: Option<LogLine>, trace: StackTraceBlock) {
        self.stats.total_exceptions += 1;

        let mut plugin_name = None;
        let mut event_name = None;
        let mut timestamp = None;
        let mut attribution = AttributionKind::Unknown;
        let mut raw_message = trace.exception_message.clone();

        if let Some(ref l) = log {
            timestamp = l.timestamp.clone();
            if raw_message.is_none() {
                raw_message = Some(l.message.clone());
            }

            if let Some(extracted) = MinecraftRules::extract_from_message(&l.message) {
                plugin_name = Some(extracted.plugin_name);
                event_name = extracted.event_name;
                attribution = AttributionKind::Confirmed;
            } else if let Some(ref src) = l.source {
                plugin_name = Some(src.clone());
                attribution = AttributionKind::Confirmed;
            }
        }

        let (hash, top_frame) = FingerprintGenerator::compute_for_trace(
            &trace.primary_exception,
            plugin_name.as_deref(),
            &trace,
        );

        if plugin_name.is_none() {
            if let Some(ref frame) = top_frame {
                let parts: Vec<&str> = frame.class_name.split('.').collect();
                if parts.len() >= 3 {
                    plugin_name = Some(parts[2].to_string());
                    attribution = AttributionKind::Inferred;
                }
            }
        }

        if let Some(ref p) = plugin_name {
            *self.plugin_error_counts.entry(p.clone()).or_insert(0) += 1;
        }

        if let Some(existing) = self.errors_by_signature.get_mut(&hash) {
            existing.record_occurrence(timestamp);
        } else {
            if let Some(limit) = self.max_signatures {
                if self.errors_by_signature.len() >= limit {
                    self.stats.dropped_signatures += 1;
                    return;
                }
            }

            let error = AggregatedError::new(
                hash,
                trace.primary_exception.clone(),
                raw_message,
                plugin_name,
                event_name,
                attribution,
                top_frame,
                timestamp,
                trace,
            );
            self.errors_by_signature.insert(hash, error);
            self.stats.unique_signatures += 1;
        }
    }

    pub fn stats(&self) -> &AggregationStats {
        &self.stats
    }

    pub fn aggregated_errors(&self) -> Vec<&AggregatedError> {
        let mut errors: Vec<&AggregatedError> = self.errors_by_signature.values().collect();
        errors.sort_by(|a, b| b.occurrences.cmp(&a.occurrences));
        errors
    }

    pub fn plugin_summary(&self) -> Vec<(&String, &usize)> {
        let mut summary: Vec<(&String, &usize)> = self.plugin_error_counts.iter().collect();
        summary.sort_by(|a, b| b.1.cmp(a.1));
        summary
    }

    pub fn to_report(&self) -> AnalysisReport<'_> {
        let mut plugins: Vec<PluginCount> = self
            .plugin_error_counts
            .iter()
            .map(|(p, c)| PluginCount {
                plugin: p.clone(),
                count: *c,
            })
            .collect();
        plugins.sort_by(|a, b| b.count.cmp(&a.count));

        AnalysisReport {
            stats: &self.stats,
            top_plugins: plugins,
            aggregated_errors: self.aggregated_errors(),
        }
    }
}
