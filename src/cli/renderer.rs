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

    // Padding Helpers

    fn visible_width(s: &str) -> usize {
        let mut in_escape = false;
        let mut len = 0;
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if c == 'm' {
                    in_escape = false;
                }
            } else {
                len += 1;
            }
        }
        len
    }

    fn pad_left(styled_text: &str, target_width: usize) -> String {
        let vlen = Self::visible_width(styled_text);
        let padding = target_width.saturating_sub(vlen);
        let mut s = String::with_capacity(styled_text.len() + padding);
        s.push_str(styled_text);
        for _ in 0..padding {
            s.push(' ');
        }
        s
    }

    fn pad_right(styled_text: &str, target_width: usize) -> String {
        let vlen = Self::visible_width(styled_text);
        let padding = target_width.saturating_sub(vlen);
        let mut s = String::with_capacity(styled_text.len() + padding);
        for _ in 0..padding {
            s.push(' ');
        }
        s.push_str(styled_text);
        s
    }

    // Header Banner

    pub fn format_banner(&self, out: &mut String) {
        let _ = writeln!(out, "{}", self.styler.cyan("╔═══════════════════════════════════════════════════════════════╗"));
        let _ = writeln!(out, "{}", self.styler.cyan("║               LOGMORPH - MINECRAFT LOG ANALYZER               ║"));
        let _ = writeln!(out, "{}", self.styler.cyan("║           Streaming Parser & StackTrace Deduplicator          ║"));
        let _ = writeln!(out, "{}", self.styler.cyan("╚═══════════════════════════════════════════════════════════════╝"));
        let _ = writeln!(out);
    }

    pub fn render_banner(&self) {
        let mut out = String::new();
        self.format_banner(&mut out);
        print!("{}", out);
    }

    // Summary Tables

    pub fn format_summary(&self, out: &mut String, stats: &AggregationStats, plugin_summary: &[(&String, &usize)]) {
        let _ = writeln!(out, "{}", self.styler.bold("=== Execution Summary ==="));
        let _ = writeln!(out, "┌─────────────────────────────┬───────────────────────────┐");
        let _ = writeln!(
            out,
            "│ {} │ {} │",
            Self::pad_left("Total Lines Processed", 27),
            Self::pad_right(&stats.total_lines.to_string(), 25)
        );
        let _ = writeln!(
            out,
            "│ {} │ {} │",
            Self::pad_left("Total Log Entries", 27),
            Self::pad_right(&stats.total_log_messages.to_string(), 25)
        );
        let _ = writeln!(
            out,
            "│ {} │ {} │",
            Self::pad_left(&self.styler.green("INFO Messages"), 27),
            Self::pad_right(&stats.info_count.to_string(), 25)
        );
        let _ = writeln!(
            out,
            "│ {} │ {} │",
            Self::pad_left(&self.styler.yellow("WARN Messages"), 27),
            Self::pad_right(&stats.warn_count.to_string(), 25)
        );
        let _ = writeln!(
            out,
            "│ {} │ {} │",
            Self::pad_left(&self.styler.red("ERROR Messages"), 27),
            Self::pad_right(&stats.error_count.to_string(), 25)
        );
        let _ = writeln!(
            out,
            "│ {} │ {} │",
            Self::pad_left(&self.styler.cyan("DEBUG Messages"), 27),
            Self::pad_right(&stats.debug_count.to_string(), 25)
        );
        let _ = writeln!(out, "├─────────────────────────────┼───────────────────────────┤");
        let _ = writeln!(
            out,
            "│ {} │ {} │",
            Self::pad_left(&self.styler.magenta("Total Exceptions Emitted"), 27),
            Self::pad_right(&stats.total_exceptions.to_string(), 25)
        );
        let _ = writeln!(
            out,
            "│ {} │ {} │",
            Self::pad_left(&self.styler.bold("Unique Error Signatures"), 27),
            Self::pad_right(&stats.unique_signatures.to_string(), 25)
        );
        if stats.dropped_signatures > 0 {
            let _ = writeln!(
                out,
                "│ {} │ {} │",
                Self::pad_left(&self.styler.red("Dropped Signatures (Cap)"), 27),
                Self::pad_right(&stats.dropped_signatures.to_string(), 25)
            );
        }
        let _ = writeln!(out, "└─────────────────────────────┴───────────────────────────┘");
        let _ = writeln!(out);

        if !plugin_summary.is_empty() {
            let _ = writeln!(out, "{}", self.styler.bold("=== Top Offending Plugins ==="));
            let _ = writeln!(out, "┌────────────────────────────────┬────────────────────────┐");
            let _ = writeln!(
                out,
                "│ {} │ {} │",
                Self::pad_left("Plugin Name", 30),
                Self::pad_right("Error Count", 22)
            );
            let _ = writeln!(out, "├────────────────────────────────┼────────────────────────┤");
            for (plugin, count) in plugin_summary {
                let display_plugin = if plugin.chars().count() > 30 {
                    format!("{}...", &plugin.chars().take(27).collect::<String>())
                } else {
                    (*plugin).clone()
                };
                let count_str = count.to_string();
                let col1 = Self::pad_left(&self.styler.yellow(&display_plugin), 30);
                let col2 = Self::pad_right(&self.styler.red(&count_str), 22);
                let _ = writeln!(out, "│ {} │ {} │", col1, col2);
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
            let _ = writeln!(out, "{}", self.styler.green("No matching error signatures found."));
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
            self.format_single_error_brief(out, err, idx + 1);
        }
    }

    fn format_single_error_brief(&self, out: &mut String, err: &AggregatedError, index: usize) {
        let occurrence_badge = self.styler.red(&format!("[Occurrences: {}]", err.occurrences));
        let hash_badge = self.styler.dim(&format!("(Signature: 0x{:016x})", err.signature_hash));

        let _ = writeln!(
            out,
            "{} {} {} {}",
            self.styler.bold(&format!("#{}", index)),
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
            let attr_info = self.styler.dim(&format!("({})", err.attribution.as_str()));
            let _ = writeln!(out, "  Plugin: {}{} {}", self.styler.yellow(plugin), event_info, attr_info);
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
                self.styler.green("-> Root Plugin Frame:"),
                self.styler.cyan(&top.class_name),
                self.styler.bold(&top.method_name),
                location
            );
        }

        let mut known = std::collections::HashSet::new();
        if let Some(ref plugin) = err.plugin_name {
            known.insert(plugin.clone());
        }

        let mut key_lines = Vec::new();
        for frame in &err.sample_trace.frames {
            let is_plugin = FrameFilter::is_plugin_frame(frame, &known);
            let loc = match (&frame.file_name, frame.line_number) {
                (Some(f), Some(l)) => format!("{}:{}", f, l),
                (Some(f), None) => f.clone(),
                _ => if frame.is_native { "Native Method".to_string() } else { "Unknown".to_string() },
            };

            if is_plugin {
                key_lines.push(format!(
                    "    {} {}.{}({})",
                    self.styler.green(">"),
                    self.styler.cyan(&frame.class_name),
                    frame.method_name,
                    loc
                ));
            }
        }

        if let Some(ref caused) = err.sample_trace.caused_by {
            let mut caused_lines = Vec::new();
            for frame in &caused.frames {
                if FrameFilter::is_plugin_frame(frame, &known) {
                    let loc = match (&frame.file_name, frame.line_number) {
                        (Some(f), Some(l)) => format!("{}:{}", f, l),
                        _ => "Unknown".to_string(),
                    };
                    caused_lines.push(format!(
                        "      {} {}.{}({})",
                        self.styler.green(">"),
                        self.styler.cyan(&frame.class_name),
                        frame.method_name,
                        loc
                    ));
                }
            }
            if !caused_lines.is_empty() {
                key_lines.push(format!("    {} {}", self.styler.magenta("Caused by:"), self.styler.red(&caused.primary_exception)));
                if let Some(ref cmsg) = caused.exception_message {
                    key_lines.push(format!("      {}", cmsg));
                }
                key_lines.extend(caused_lines);
            }
        }

        if !key_lines.is_empty() {
            let _ = writeln!(out, "  Stack Trace (Key Frames):");
            for kl in key_lines {
                let _ = writeln!(out, "{}", kl);
            }
        }

        let _ = writeln!(out);
    }

    // Single Error Inspection

    pub fn format_single_error_detailed(&self, out: &mut String, err: &AggregatedError, index: usize) {
        let _ = writeln!(out, "{}", self.styler.bold(&format!("=== Detailed Inspection: Error Signature #{} ===", index)));
        let _ = writeln!(out);

        let occurrence_badge = self.styler.red(&format!("[Occurrences: {}]", err.occurrences));
        let hash_badge = self.styler.dim(&format!("(Signature: 0x{:016x})", err.signature_hash));

        let _ = writeln!(
            out,
            "Primary Exception: {} {} {}",
            self.styler.bold(&self.styler.red(&err.primary_exception)),
            occurrence_badge,
            hash_badge
        );

        if let Some(ref plugin) = err.plugin_name {
            let event_info = err
                .event_name
                .as_deref()
                .map(|ev| format!(" (Event: {})", self.styler.cyan(ev)))
                .unwrap_or_default();
            let _ = writeln!(out, "Plugin Attribution: {}{} [{}]", self.styler.yellow(plugin), event_info, err.attribution.as_str());
        }

        let msg_display = err.exception_message.as_deref().unwrap_or("<not available>");
        let _ = writeln!(out, "Message: {}", msg_display);

        if let (Some(first), Some(last)) = (&err.first_seen, &err.last_seen) {
            let _ = writeln!(out, "Timeline: First seen at {}, last seen at {}", first, last);
        }

        if let Some(ref top) = err.top_plugin_frame {
            let location = match (&top.file_name, top.line_number) {
                (Some(f), Some(l)) => format!("{}:{}", f, l),
                _ => "Unknown".to_string(),
            };
            let _ = writeln!(
                out,
                "Root Plugin Frame: {}.{}({})",
                self.styler.cyan(&top.class_name),
                self.styler.bold(&top.method_name),
                location
            );
        }

        let mut known = std::collections::HashSet::new();
        if let Some(ref plugin) = err.plugin_name {
            known.insert(plugin.clone());
        }

        let has_frames = !err.sample_trace.frames.is_empty();
        let has_caused_by = err.sample_trace.caused_by.is_some();

        let _ = writeln!(out);
        if has_frames || has_caused_by {
            let _ = writeln!(out, "Full Stack Trace:");
            for frame in &err.sample_trace.frames {
                let kind = FrameFilter::classify_frame(frame, &known);
                let loc = match (&frame.file_name, frame.line_number) {
                    (Some(f), Some(l)) => format!("{}:{}", f, l),
                    _ => if frame.is_native { "Native Method".to_string() } else { "Unknown".to_string() },
                };

                match kind {
                    crate::engine::frame_filter::FrameKind::Plugin => {
                        let _ = writeln!(
                            out,
                            "  {} {}.{}({})",
                            self.styler.green("> [Plugin]"),
                            self.styler.cyan(&frame.class_name),
                            frame.method_name,
                            loc
                        );
                    }
                    crate::engine::frame_filter::FrameKind::Framework => {
                        let _ = writeln!(
                            out,
                            "    {} {}.{}({})",
                            self.styler.dim("[Framework]"),
                            self.styler.dim(&frame.class_name),
                            self.styler.dim(&frame.method_name),
                            self.styler.dim(&loc)
                        );
                    }
                    crate::engine::frame_filter::FrameKind::Unknown => {
                        let _ = writeln!(
                            out,
                            "    {} {}.{}({})",
                            self.styler.yellow("[Unknown]"),
                            self.styler.yellow(&frame.class_name),
                            frame.method_name,
                            loc
                        );
                    }
                }
            }

            if let Some(ref caused) = err.sample_trace.caused_by {
                let _ = writeln!(out);
                let _ = writeln!(out, "Caused By: {}", self.styler.red(&caused.primary_exception));
                if let Some(ref cmsg) = caused.exception_message {
                    let _ = writeln!(out, "  Message: {}", cmsg);
                }
                for frame in &caused.frames {
                    let kind = FrameFilter::classify_frame(frame, &known);
                    let loc = match (&frame.file_name, frame.line_number) {
                        (Some(f), Some(l)) => format!("{}:{}", f, l),
                        _ => "Unknown".to_string(),
                    };
                    match kind {
                        crate::engine::frame_filter::FrameKind::Plugin => {
                            let _ = writeln!(
                                out,
                                "  {} {}.{}({})",
                                self.styler.green("> [Plugin]"),
                                self.styler.cyan(&frame.class_name),
                                frame.method_name,
                                loc
                            );
                        }
                        crate::engine::frame_filter::FrameKind::Framework => {
                            let _ = writeln!(
                                out,
                                "    {} {}.{}({})",
                                self.styler.dim("[Framework]"),
                                self.styler.dim(&frame.class_name),
                                self.styler.dim(&frame.method_name),
                                self.styler.dim(&loc)
                            );
                        }
                        crate::engine::frame_filter::FrameKind::Unknown => {
                            let _ = writeln!(
                                out,
                                "    {} {}.{}({})",
                                self.styler.yellow("[Unknown]"),
                                self.styler.yellow(&frame.class_name),
                                frame.method_name,
                                loc
                            );
                        }
                    }
                }
            }
        } else {
            let _ = writeln!(out, "Full Stack Trace: {}", self.styler.dim("<no stack frames recorded in log>"));
        }

        let _ = writeln!(out);
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

    // Log Line Formatter

    pub fn format_log_line(&self, out: &mut String, line: &crate::models::log_line::LogLine) {
        if let Some(ref ts) = line.timestamp {
            let _ = write!(out, "[{}] ", self.styler.dim(ts));
        }
        let level_str = match line.level {
            crate::models::log_line::LogLevel::Info => self.styler.green(line.level.as_str()),
            crate::models::log_line::LogLevel::Warn => self.styler.yellow(line.level.as_str()),
            crate::models::log_line::LogLevel::Error => self.styler.red(line.level.as_str()),
            crate::models::log_line::LogLevel::Debug => self.styler.cyan(line.level.as_str()),
            crate::models::log_line::LogLevel::Trace => self.styler.dim(line.level.as_str()),
            crate::models::log_line::LogLevel::Unknown => self.styler.dim(line.level.as_str()),
        };
        let _ = write!(out, "[{}] ", level_str);
        if let Some(ref src) = line.source {
            let _ = write!(out, "[{}] ", self.styler.yellow(src));
        }
        let _ = writeln!(out, "{}", line.message);
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

// Unit Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_width_calculation() {
        let plain = "INFO Messages";
        assert_eq!(TerminalRenderer::visible_width(plain), 13);

        let colored = "\x1b[32mINFO Messages\x1b[0m";
        assert_eq!(TerminalRenderer::visible_width(colored), 13);

        let bold_colored = "\x1b[1m\x1b[31mERROR Messages\x1b[0m";
        assert_eq!(TerminalRenderer::visible_width(bold_colored), 14);
    }

    #[test]
    fn test_banner_line_lengths() {
        let renderer = TerminalRenderer::new(false);
        let mut banner = String::new();
        renderer.format_banner(&mut banner);

        let lines: Vec<&str> = banner.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 4);
        for line in lines {
            assert_eq!(TerminalRenderer::visible_width(line), 65);
        }
    }

    #[test]
    fn test_summary_table_alignment() {
        let renderer = TerminalRenderer::new(false);
        let stats = AggregationStats {
            total_lines: 1724,
            total_log_messages: 1473,
            info_count: 1273,
            warn_count: 186,
            error_count: 14,
            debug_count: 0,
            total_exceptions: 6,
            unique_signatures: 4,
            dropped_signatures: 0,
        };
        let p1 = "PlaceholderAPI".to_string();
        let p2 = "EvenMoreFish".to_string();
        let c1 = 2usize;
        let c2 = 1usize;
        let plugins = vec![(&p1, &c1), (&p2, &c2)];

        let mut out = String::new();
        renderer.format_summary(&mut out, &stats, &plugins);

        let lines: Vec<&str> = out.lines().collect();
        // Check Execution Summary box
        for line in &lines[1..12] {
            assert_eq!(TerminalRenderer::visible_width(line), 59);
        }
        // Check Top Offending Plugins box
        for line in &lines[14..19] {
            assert_eq!(TerminalRenderer::visible_width(line), 59);
        }
    }
}

