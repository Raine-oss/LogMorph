// CLI Arguments

use clap::Parser;
use std::path::PathBuf;

// Command Line Options

#[derive(Parser, Debug)]
#[command(name = "logmorph")]
#[command(author = "LogMorph Team")]
#[command(version = "0.1.0")]
#[command(about = "High-performance Minecraft & Java log parser and stack trace aggregator", long_about = None)]
pub struct CliArgs {
    #[arg(value_name = "FILE", help = "Path to the log file (reads from stdin if omitted)")]
    pub file: Option<PathBuf>,

    #[arg(short = 'l', long = "level", value_name = "LEVEL", help = "Filter by log level (INFO, WARN, ERROR, DEBUG)")]
    pub level: Option<String>,

    #[arg(short = 'p', long = "plugin", value_name = "PLUGIN", help = "Filter by plugin name")]
    pub plugin: Option<String>,

    #[arg(short = 's', long = "summary-only", help = "Display only the summary tables")]
    pub summary_only: bool,

    #[arg(long = "no-color", help = "Disable ANSI color output")]
    pub no_color: bool,
}
