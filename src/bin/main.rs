// CLI Main

use clap::Parser;
use logmorph::cli::{CliArgs, TerminalRenderer};
use logmorph::engine::AggregationEngine;
use logmorph::models::LogLevel;
use logmorph::parser::{LogStreamReader, ParsedEvent};
use std::process::ExitCode;

// Application Runner

fn main() -> ExitCode {
    let args = CliArgs::parse();
    let renderer = TerminalRenderer::new(args.no_color);

    let mut engine = AggregationEngine::new();

    let target_level = args.level.as_deref().map(LogLevel::from_str);

    let result = if let Some(ref path) = args.file {
        if !path.exists() {
            eprintln!("Error: File not found: {}", path.display());
            return ExitCode::FAILURE;
        }

        match LogStreamReader::from_path(path) {
            Ok(reader) => process_stream(reader, &mut engine, target_level.as_ref()),
            Err(e) => {
                eprintln!("Error opening file {}: {}", path.display(), e);
                return ExitCode::FAILURE;
            }
        }
    } else {
        let reader = LogStreamReader::from_stdin();
        process_stream(reader, &mut engine, target_level.as_ref())
    };

    if let Err(e) = result {
        eprintln!("Error reading log stream: {}", e);
        return ExitCode::FAILURE;
    }

    renderer.render_all(&engine, args.summary_only, args.plugin.as_deref());

    ExitCode::SUCCESS
}

// Stream Processor

fn process_stream<R: std::io::BufRead>(
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
