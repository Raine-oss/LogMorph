// CLI Renderer

use crate::engine::aggregation::{AggregationEngine, AggregationStats};
use crate::engine::frame_filter::FrameFilter;
use crate::models::error_event::AggregatedError;
use std::fmt::Write;

// Color Formatter

struct Styler {
    enabled: bool,
}

impl Styler {
    fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    fn bold(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[1m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    fn red(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[31m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    fn green(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[32m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    fn yellow(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[33m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    fn cyan(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[36m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    fn magenta(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[35m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    fn dim(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[2m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }
}

// Terminal Output Renderer

pub struct TerminalRenderer {
    styler: Styler,
}

impl TerminalRenderer {
    pub fn new(no_color: bool) -> Self {
        Self {
            styler: Styler::new(!no_color),
        }
    }

    // Header Banner

    pub fn format_banner(&self, out: &mut String) {
        let _ = writeln!(out, "{}", self.styler.cyan("╔═══════════════════════════════════════════════════════════════╗"));
        let _ = writeln!(out, "{}", self.styler.cyan("║              LOGMORPH - MINECRAFT LOG ANALYZER               ║"));
        let _ = writeln!(out, "{}", self.styler.cyan("║           Streaming Parser & StackTrace Deduplicator         ║"));
        let _ = writeln!(out, "{}", self.styler.cyan("╚═══════════════════════════════════════════════════════════════╝"));
        let _ = writeln!(out);
    }

    // Summary Tables

    pub fn format_summary(&self, out: &mut String, stats: &AggregationStats, plugin_summary: &[(&String, &usize)]) {
        let _ = writeln!(out, "{}", self.styler.bold("=== Execution Summary ==="));
        let _ = writeln!(out, "┌─────────────────────────────┬───────────────────────────┐");
        let _ = writeln!(out, "│ {:<27} │ {:>25} │", "Total Lines Processed", stats.total_lines);
        let _ = writeln!(out, "│ {:<27} │ {:>25} │", "Total Log Entries", stats.total_log_messages);
        let _ = writeln!(out, "│ {:<27} │ {:>25} │", self.styler.green("INFO Messages"), stats.info_count);
        let _ = writeln!(out, "│ {:<27} │ {:>25} │", self.styler.yellow("WARN Messages"), stats.warn_count);
        let _ = writeln!(out, "│ {:<27} │ {:>25} │", self.styler.red("ERROR Messages"), stats.error_count);
        let _ = writeln!(out, "│ {:<27} │ {:>25} │", self.styler.cyan("DEBUG Messages"), stats.debug_count);
        let _ = writeln!(out, "├─────────────────────────────┼───────────────────────────┤");
        let _ = writeln!(out, "│ {:<27} │ {:>25} │", self.styler.magenta("Total Exceptions Emitted"), stats.total_exceptions);
        let _ = writeln!(out, "│ {:<27} │ {:>25} │", self.styler.bold("Unique Error Signatures"), stats.unique_signatures);
        let _ = writeln!(out, "└─────────────────────────────┴───────────────────────────┘");
        let _ = writeln!(out);

        if !plugin_summary.is_empty() {
            let _ = writeln!(out, "{}", self.styler.bold("=== Top Offending Plugins ==="));
            let _ = writeln!(out, "┌────────────────────────────────┬────────────────────────┐");
            let _ = writeln!(out, "│ {:<30} │ {:>22} │", "Plugin Name", "Error Count");
            let _ = writeln!(out, "├────────────────────────────────┼────────────────────────┤");
            for (plugin, count) in plugin_summary {
                let _ = writeln!(
                    out,
                    "│ {:<30} │ {:>22} │",
                    self.styler.yellow(plugin),
                    self.styler.red(&count.to_string())
                );
            }
            let _ = writeln!(out, "└────────────────────────────────┴────────────────────────┘");
            let _ = writeln!(out);
        }
    }

    // Aggregated Errors

    pub fn format_errors(
        &self,
        out: &mut String,
        errors: &[&AggregatedError],
        plugin_filter: Option<&str>,
    ) {
        let filtered: Vec<&&AggregatedError> = errors
            .iter()
            .filter(|e| {
                if let Some(target_plugin) = plugin_filter {
                    e.plugin_name
                        .as_ref()
                        .map(|p| p.eq_ignore_ascii_case(target_plugin))
                        .unwrap_or(false)
                } else {
                    true
                }
            })
            .collect();

        if filtered.is_empty() {
            let _ = writeln!(out, "{}", self.styler.green("✔ No matching error signatures found."));
            return;
        }

        let _ = writeln!(
            out,
            "{}",
            self.styler.bold(&format!(
                "=== Aggregated Error Signatures ({}) ===",
                filtered.len()
            ))
        );
        let _ = writeln!(out);

        for (idx, err) in filtered.iter().enumerate() {
            let occurrence_badge = self.styler.red(&format!("[Occurrences: {}]", err.occurrences));
            let hash_badge = self.styler.dim(&format!("(Signature: 0x{:016x})", err.signature_hash));

            let _ = writeln!(
                out,
                "{} {} {} {}",
                self.styler.bold(&format!("#{}", idx + 1)),
                occurrence_badge,
                self.styler.bold(&self.styler.red(&err.primary_exception)),
                hash_badge
            );

            if let Some(ref plugin) = err.plugin_name {
                let event_info = err
                    .event_name
                    .as_deref()
                    .map(|ev| format!(" on Event {}", self.styler.cyan(ev)))
                    .unwrap_or_default();
                let _ = writeln!(out, "  Plugin: {}{}", self.styler.yellow(plugin), event_info);
            }

            if let Some(ref msg) = err.exception_message {
                let _ = writeln!(out, "  Message: {}", msg);
            }

            if let (Some(first), Some(last)) = (&err.first_seen, &err.last_seen) {
                if first == last {
                    let _ = writeln!(out, "  Timestamp: {}", self.styler.dim(first));
                } else {
                    let _ = writeln!(out, "  Timeline: {} -> {}", self.styler.dim(first), self.styler.dim(last));
                }
            }

            if let Some(ref top) = err.top_plugin_frame {
                let location = match (&top.file_name, top.line_number) {
                    (Some(f), Some(l)) => format!("{}:{}", f, l),
                    (Some(f), None) => f.clone(),
                    _ => "Unknown source".to_string(),
                };
                let _ = writeln!(
                    out,
                    "  {} {}.{}({})",
                    self.styler.green("↳ Root Plugin Frame:"),
                    self.styler.cyan(&top.class_name),
                    self.styler.bold(&top.method_name),
                    location
                );
            }

            let _ = writeln!(out, "  Stack Trace (Key Frames):");
            let mut plugin_frames_printed = 0;
            for frame in &err.sample_trace.frames {
                let is_plugin = FrameFilter::is_plugin_frame(frame);
                let loc = match (&frame.file_name, frame.line_number) {
                    (Some(f), Some(l)) => format!("{}:{}", f, l),
                    (Some(f), None) => f.clone(),
                    _ => if frame.is_native { "Native Method".to_string() } else { "Unknown".to_string() },
                };

                if is_plugin {
                    plugin_frames_printed += 1;
                    let _ = writeln!(
                        out,
                        "    {} {}.{}({})",
                        self.styler.green("▶"),
                        self.styler.cyan(&frame.class_name),
                        frame.method_name,
                        loc
                    );
                } else if plugin_frames_printed < 2 {
                    let _ = writeln!(
                        out,
                        "      {} {}.{}({})",
                        self.styler.dim("·"),
                        self.styler.dim(&frame.class_name),
                        self.styler.dim(&frame.method_name),
                        self.styler.dim(&loc)
                    );
                }
            }

            if let Some(ref caused) = err.sample_trace.caused_by {
                let _ = writeln!(out, "    {} {}", self.styler.magenta("Caused by:"), self.styler.red(&caused.primary_exception));
                if let Some(ref cmsg) = caused.exception_message {
                    let _ = writeln!(out, "      {}", cmsg);
                }
                for frame in &caused.frames {
                    if FrameFilter::is_plugin_frame(frame) {
                        let loc = match (&frame.file_name, frame.line_number) {
                            (Some(f), Some(l)) => format!("{}:{}", f, l),
                            _ => "Unknown".to_string(),
                        };
                        let _ = writeln!(
                            out,
                            "      {} {}.{}({})",
                            self.styler.green("▶"),
                            self.styler.cyan(&frame.class_name),
                            frame.method_name,
                            loc
                        );
                    }
                }
            }

            let _ = writeln!(out);
        }
    }

    // Format All Output

    pub fn format_all(
        &self,
        engine: &AggregationEngine,
        summary_only: bool,
        plugin_filter: Option<&str>,
    ) -> String {
        let mut out = String::with_capacity(4096);
        self.format_banner(&mut out);
        self.format_summary(&mut out, engine.stats(), &engine.plugin_summary());
        if !summary_only {
            self.format_errors(&mut out, &engine.aggregated_errors(), plugin_filter);
        }
        out
    }

    // Render All Output

    pub fn render_all(
        &self,
        engine: &AggregationEngine,
        summary_only: bool,
        plugin_filter: Option<&str>,
    ) {
        let output = self.format_all(engine, summary_only, plugin_filter);
        print!("{}", output);
    }
}
