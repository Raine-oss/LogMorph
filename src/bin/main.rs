// CLI Main

use clap::Parser;
use logmorph::cli::args::{CliArgs, Commands, OutputFormat};
use logmorph::cli::TerminalRenderer;
use logmorph::engine::AggregationEngine;
use logmorph::models::LogLevel;
use logmorph::parser::{LogStreamReader, ParsedEvent};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::process::ExitCode;
use std::thread::sleep;
use std::time::Duration;

// Exit Codes

pub struct ExitStatus;

impl ExitStatus {
    pub const SUCCESS: u8 = 0;
    pub const FILE_NOT_FOUND: u8 = 1;
    pub const INVALID_INPUT: u8 = 2;
    pub const INTERNAL_ERROR: u8 = 3;

    pub fn success() -> ExitCode {
        ExitCode::from(Self::SUCCESS)
    }

    pub fn file_not_found() -> ExitCode {
        ExitCode::from(Self::FILE_NOT_FOUND)
    }

    pub fn invalid_input() -> ExitCode {
        ExitCode::from(Self::INVALID_INPUT)
    }

    pub fn internal_error() -> ExitCode {
        ExitCode::from(Self::INTERNAL_ERROR)
    }
}

// Application Runner

fn main() -> ExitCode {
    let args = CliArgs::parse();

    match args.command {
        Some(Commands::Analyze {
            file,
            level,
            plugin,
            summary_only,
            format,
            max_signatures,
            no_color,
        }) => run_analyze(
            file.as_deref(),
            level.as_deref(),
            plugin.as_deref(),
            summary_only,
            format,
            max_signatures,
            no_color,
        ),

        Some(Commands::Summary {
            file,
            format,
            no_color,
        }) => run_analyze(
            file.as_deref(),
            None,
            None,
            true,
            format,
            None,
            no_color,
        ),

        Some(Commands::Inspect {
            file,
            error_index,
            no_color,
        }) => run_inspect(file.as_deref(), error_index, no_color),

        Some(Commands::Export {
            file,
            output,
            format,
        }) => run_export(file.as_deref(), output.as_deref(), format),

        Some(Commands::Watch {
            file,
            level,
            plugin,
            no_color,
        }) => run_watch(&file, level.as_deref(), plugin.as_deref(), no_color),

        None => run_analyze(
            args.file.as_deref(),
            args.level.as_deref(),
            args.plugin.as_deref(),
            args.summary_only,
            args.format,
            args.max_signatures,
            args.no_color,
        ),
    }
}

// Analyze Runner

fn run_analyze(
    file_path: Option<&Path>,
    level: Option<&str>,
    plugin: Option<&str>,
    summary_only: bool,
    format: OutputFormat,
    max_signatures: Option<usize>,
    no_color: bool,
) -> ExitCode {
    let mut engine = AggregationEngine::with_max_signatures(max_signatures);
    let target_level = level.map(LogLevel::from_str);

    let result = if let Some(path) = file_path {
        if !path.exists() {
            eprintln!("Error: File not found: {}", path.display());
            return ExitStatus::file_not_found();
        }
        match LogStreamReader::from_path(path) {
            Ok(reader) => process_stream(reader, &mut engine, target_level.as_ref()),
            Err(e) => {
                eprintln!("Error opening file {}: {}", path.display(), e);
                return ExitStatus::file_not_found();
            }
        }
    } else {
        let reader = LogStreamReader::from_stdin();
        process_stream(reader, &mut engine, target_level.as_ref())
    };

    if let Err(e) = result {
        eprintln!("Error reading log stream: {}", e);
        return ExitStatus::internal_error();
    }

    match format {
        OutputFormat::Json => {
            let report = engine.to_report();
            match serde_json::to_string_pretty(&report) {
                Ok(json) => println!("{}", json),
                Err(e) => {
                    eprintln!("Error serializing JSON: {}", e);
                    return ExitStatus::internal_error();
                }
            }
        }
        OutputFormat::Text => {
            let renderer = TerminalRenderer::new(no_color);
            renderer.render_all(&engine, summary_only, plugin);
        }
    }

    ExitStatus::success()
}

// Export Runner

fn run_export(
    file_path: Option<&Path>,
    output_path: Option<&Path>,
    format: OutputFormat,
) -> ExitCode {
    let mut engine = AggregationEngine::new();

    let result = if let Some(path) = file_path {
        if !path.exists() {
            eprintln!("Error: File not found: {}", path.display());
            return ExitStatus::file_not_found();
        }
        match LogStreamReader::from_path(path) {
            Ok(reader) => process_stream(reader, &mut engine, None),
            Err(e) => {
                eprintln!("Error opening file {}: {}", path.display(), e);
                return ExitStatus::file_not_found();
            }
        }
    } else {
        let reader = LogStreamReader::from_stdin();
        process_stream(reader, &mut engine, None)
    };

    if let Err(e) = result {
        eprintln!("Error reading log stream: {}", e);
        return ExitStatus::internal_error();
    }

    let report = engine.to_report();
    let json_content = match serde_json::to_string_pretty(&report) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error serializing JSON: {}", e);
            return ExitStatus::internal_error();
        }
    };

    if let Some(out_file) = output_path {
        if let Err(e) = fs::write(out_file, json_content) {
            eprintln!("Error writing export to {}: {}", out_file.display(), e);
            return ExitStatus::internal_error();
        }
        println!("Successfully exported analysis to: {}", out_file.display());
    } else {
        match format {
            OutputFormat::Json => println!("{}", json_content),
            OutputFormat::Text => {
                let renderer = TerminalRenderer::new(true);
                renderer.render_all(&engine, false, None);
            }
        }
    }

    ExitStatus::success()
}

// Inspect Runner

fn run_inspect(file_path: Option<&Path>, error_index: usize, no_color: bool) -> ExitCode {
    if error_index == 0 {
        eprintln!("Error: Error index must be 1 or greater.");
        return ExitStatus::invalid_input();
    }

    let mut engine = AggregationEngine::new();

    let result = if let Some(path) = file_path {
        if !path.exists() {
            eprintln!("Error: File not found: {}", path.display());
            return ExitStatus::file_not_found();
        }
        match LogStreamReader::from_path(path) {
            Ok(reader) => process_stream(reader, &mut engine, None),
            Err(e) => {
                eprintln!("Error opening file {}: {}", path.display(), e);
                return ExitStatus::file_not_found();
            }
        }
    } else {
        let reader = LogStreamReader::from_stdin();
        process_stream(reader, &mut engine, None)
    };

    if let Err(e) = result {
        eprintln!("Error reading log stream: {}", e);
        return ExitStatus::internal_error();
    }

    let errors = engine.aggregated_errors();
    if error_index > errors.len() {
        eprintln!(
            "Error: Specified index #{} out of range. Total unique error signatures: {}",
            error_index,
            errors.len()
        );
        return ExitStatus::invalid_input();
    }

    let target_error = errors[error_index - 1];
    let renderer = TerminalRenderer::new(no_color);
    let mut output = String::new();
    renderer.format_single_error_detailed(&mut output, target_error, error_index);
    print!("{}", output);

    ExitStatus::success()
}

// Watch Runner

fn run_watch(
    file_path: &Path,
    level: Option<&str>,
    plugin: Option<&str>,
    no_color: bool,
) -> ExitCode {
    if !file_path.exists() {
        eprintln!("Error: File not found: {}", file_path.display());
        return ExitStatus::file_not_found();
    }

    let renderer = TerminalRenderer::new(no_color);
    renderer.render_banner();
    println!("Watching {} for new events (Ctrl+C to stop)...", file_path.display());
    println!();

    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file {}: {}", file_path.display(), e);
            return ExitStatus::file_not_found();
        }
    };

    let mut reader = BufReader::new(file);
    let mut current_pos = reader.seek(SeekFrom::End(0)).unwrap_or(0);

    let mut parser = logmorph::parser::StateMachineParser::new();
    let mut engine = AggregationEngine::new();
    let mut line_buf = String::new();
    let target_level = level.map(LogLevel::from_str);

    loop {
        line_buf.clear();
        match reader.read_line(&mut line_buf) {
            Ok(0) => {
                if let Ok(meta) = fs::metadata(file_path) {
                    if meta.len() < current_pos {
                        let _ = reader.seek(SeekFrom::Start(0));
                        current_pos = 0;
                    }
                }
                sleep(Duration::from_millis(250));
            }
            Ok(bytes_read) => {
                current_pos += bytes_read as u64;
                let trimmed = line_buf.trim_end_matches(&['\r', '\n'][..]);
                let events = parser.process_line(trimmed);
                for event in events {
                    let should_display = match &event {
                        ParsedEvent::Line(l) => {
                            if let Some(ref t) = target_level {
                                &l.level == t
                            } else {
                                true
                            }
                        }
                        ParsedEvent::ErrorWithTrace { log, .. } => {
                            if let Some(ref t) = target_level {
                                log.as_ref().map(|l| &l.level == t).unwrap_or(true)
                            } else {
                                true
                            }
                        }
                        ParsedEvent::Raw(_) => false,
                    };

                    if should_display {
                        engine.feed_event(event);
                        let latest = engine.aggregated_errors();
                        if let Some(err) = latest.first() {
                            if let Some(target_p) = plugin {
                                let matches = err.plugin_name.as_deref().map(|p| p.eq_ignore_ascii_case(target_p)).unwrap_or(false);
                                if !matches {
                                    continue;
                                }
                            }
                            let mut buf = String::new();
                            renderer.format_single_error_detailed(&mut buf, err, 1);
                            print!("{}", buf);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading stream: {}", e);
                return ExitStatus::internal_error();
            }
        }
    }
}

// Stream Processor

fn process_stream<R: BufRead>(
    stream: LogStreamReader<R>,
    engine: &mut AggregationEngine,
    level_filter: Option<&LogLevel>,
) -> std::io::Result<()> {
    for event_res in stream {
        let event = event_res?;

        if let Some(target) = level_filter {
            match &event {
                ParsedEvent::Line(line) => {
                    if &line.level != target {
                        continue;
                    }
                }
                ParsedEvent::ErrorWithTrace { log, .. } => {
                    if let Some(line) = log {
                        if &line.level != target {
                            continue;
                        }
                    }
                }
                ParsedEvent::Raw(_) => continue,
            }
        }

        engine.feed_event(event);
    }

    Ok(())
}
