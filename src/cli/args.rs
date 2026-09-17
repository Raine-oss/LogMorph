// CLI Arguments

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

// Output Format

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

// Subcommands

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Analyze a log file and display aggregated errors")]
    Analyze {
        #[arg(value_name = "FILE", help = "Path to the log file (reads from stdin if omitted)")]
        file: Option<PathBuf>,

        #[arg(short = 'l', long = "level", value_name = "LEVEL", help = "Filter by log level (INFO, WARN, ERROR, DEBUG)")]
        level: Option<String>,

        #[arg(short = 'p', long = "plugin", value_name = "PLUGIN", help = "Filter by plugin name")]
        plugin: Option<String>,

        #[arg(short = 's', long = "summary-only", help = "Display only the summary tables")]
        summary_only: bool,

        #[arg(long = "format", value_enum, default_value_t = OutputFormat::Text, help = "Output format")]
        format: OutputFormat,

        #[arg(long = "max-signatures", value_name = "N", help = "Cap maximum tracked error signatures in memory")]
        max_signatures: Option<usize>,

        #[arg(long = "no-color", help = "Disable ANSI color output")]
        no_color: bool,
    },

    #[command(about = "Display execution summary tables only")]
    Summary {
        #[arg(value_name = "FILE", help = "Path to the log file (reads from stdin if omitted)")]
        file: Option<PathBuf>,

        #[arg(long = "format", value_enum, default_value_t = OutputFormat::Text, help = "Output format")]
        format: OutputFormat,

        #[arg(long = "no-color", help = "Disable ANSI color output")]
        no_color: bool,
    },

    #[command(name = "inspect", alias = "show", about = "Inspect full stack trace and details for a specific error signature")]
    Inspect {
        #[arg(value_name = "FILE", help = "Path to the log file (reads from stdin if omitted)")]
        file: Option<PathBuf>,

        #[arg(short = 'e', long = "error", required = true, value_name = "INDEX", help = "1-based index of the error signature to inspect")]
        error_index: usize,

        #[arg(long = "no-color", help = "Disable ANSI color output")]
        no_color: bool,
    },

    #[command(about = "Export structured analysis data to a file or stdout")]
    Export {
        #[arg(value_name = "FILE", help = "Path to the log file (reads from stdin if omitted)")]
        file: Option<PathBuf>,

        #[arg(short = 'o', long = "output", value_name = "OUTPUT_FILE", help = "Path to write output file (prints to stdout if omitted)")]
        output: Option<PathBuf>,

        #[arg(long = "format", value_enum, default_value_t = OutputFormat::Json, help = "Export format (json)")]
        format: OutputFormat,
    },

    #[command(about = "Monitor a log file in real-time as lines arrive")]
    Watch {
        #[arg(value_name = "FILE", help = "Path to the log file to follow")]
        file: PathBuf,

        #[arg(short = 'l', long = "level", value_name = "LEVEL", help = "Filter by log level")]
        level: Option<String>,

        #[arg(short = 'p', long = "plugin", value_name = "PLUGIN", help = "Filter by plugin name")]
        plugin: Option<String>,

        #[arg(long = "no-color", help = "Disable ANSI color output")]
        no_color: bool,
    },
}

// Command Line Options

#[derive(Parser, Debug)]
#[command(name = "logmorph")]
#[command(author = "LogMorph Team")]
#[command(version = "0.1.0")]
#[command(about = "High-performance Minecraft & Java log parser and stack trace deduplicator")]
pub struct CliArgs {
    #[arg(value_name = "FILE", help = "Path to the log file (reads from stdin if omitted)")]
    pub file: Option<PathBuf>,

    #[arg(short = 'l', long = "level", value_name = "LEVEL", help = "Filter by log level (INFO, WARN, ERROR, DEBUG)")]
    pub level: Option<String>,

    #[arg(short = 'p', long = "plugin", value_name = "PLUGIN", help = "Filter by plugin name")]
    pub plugin: Option<String>,

    #[arg(short = 's', long = "summary-only", help = "Display only the summary tables")]
    pub summary_only: bool,

    #[arg(long = "format", value_enum, default_value_t = OutputFormat::Text, help = "Output format")]
    pub format: OutputFormat,

    #[arg(long = "max-signatures", value_name = "N", help = "Cap maximum tracked error signatures in memory")]
    pub max_signatures: Option<usize>,

    #[arg(long = "no-color", help = "Disable ANSI color output")]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}
