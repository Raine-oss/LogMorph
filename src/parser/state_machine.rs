// State Machine

use crate::models::log_line::LogLine;
use crate::models::stack_trace::{StackFrame, StackTraceBlock};
use crate::parser::line_parser::{LineParser, ParsedLine};

// Parsed Event

#[derive(Debug, PartialEq, Eq)]
pub enum ParsedEvent {
    Line(LogLine),
    ErrorWithTrace {
        log: Option<LogLine>,
        trace: StackTraceBlock,
        line_count: usize,
    },
    Raw(String),
}

// Internal Trace Builder

struct TraceBuilder {
    exception: String,
    message: Option<String>,
    frames: Vec<StackFrame>,
    caused_by: Option<Box<TraceBuilder>>,
    lines_consumed: usize,
}

impl TraceBuilder {
    fn new(exception: String, message: Option<String>, initial_lines: usize) -> Self {
        Self {
            exception,
            message,
            frames: Vec::new(),
            caused_by: None,
            lines_consumed: initial_lines,
        }
    }

    fn add_frame(&mut self, frame: StackFrame) {
        self.lines_consumed += 1;
        if let Some(ref mut child) = self.caused_by {
            child.add_frame(frame);
        } else {
            self.frames.push(frame);
        }
    }

    fn add_caused_by(&mut self, exception: String, message: Option<String>) {
        self.lines_consumed += 1;
        if let Some(ref mut child) = self.caused_by {
            child.add_caused_by(exception, message);
        } else {
            self.caused_by = Some(Box::new(TraceBuilder::new(exception, message, 0)));
        }
    }

    fn record_more(&mut self) {
        self.lines_consumed += 1;
    }

    fn build(self) -> (StackTraceBlock, usize) {
        let lines = self.lines_consumed;
        let block = StackTraceBlock {
            primary_exception: self.exception,
            exception_message: self.message,
            frames: self.frames,
            caused_by: self.caused_by.map(|c| Box::new(c.build().0)),
        };
        (block, lines)
    }
}

// Parser State

enum State {
    Idle,
    PendingLog(LogLine),
    AccumulatingStackTrace {
        log: Option<LogLine>,
        builder: TraceBuilder,
    },
}

// State Machine Parser

pub struct StateMachineParser {
    state: State,
}

impl StateMachineParser {
    pub fn new() -> Self {
        Self { state: State::Idle }
    }

    pub fn process_line(&mut self, line: &str) -> Vec<ParsedEvent> {
        let mut events = Vec::new();
        let parsed = LineParser::parse(line);

        match std::mem::replace(&mut self.state, State::Idle) {
            State::Idle => match parsed {
                ParsedLine::Log(log_line) => {
                    self.state = State::PendingLog(log_line);
                }
                ParsedLine::ExceptionHeader { exception, message } => {
                    self.state = State::AccumulatingStackTrace {
                        log: None,
                        builder: TraceBuilder::new(exception, message, 1),
                    };
                }
                ParsedLine::Text(text) => {
                    events.push(ParsedEvent::Raw(text));
                }
                ParsedLine::Frame(_) | ParsedLine::CausedBy { .. } | ParsedLine::MoreFrames(_) => {
                    events.push(ParsedEvent::Raw(line.to_string()));
                }
            },
            State::PendingLog(prev_log) => match parsed {
                ParsedLine::ExceptionHeader { exception, message } => {
                    self.state = State::AccumulatingStackTrace {
                        log: Some(prev_log),
                        builder: TraceBuilder::new(exception, message, 2),
                    };
                }
                ParsedLine::Frame(frame) => {
                    let mut builder = TraceBuilder::new(
                        "java.lang.Exception".to_string(),
                        Some(prev_log.message.clone()),
                        1,
                    );
                    builder.add_frame(frame);
                    self.state = State::AccumulatingStackTrace {
                        log: Some(prev_log),
                        builder,
                    };
                }
                ParsedLine::Log(new_log) => {
                    events.push(ParsedEvent::Line(prev_log));
                    self.state = State::PendingLog(new_log);
                }
                ParsedLine::Text(text) => {
                    events.push(ParsedEvent::Line(prev_log));
                    events.push(ParsedEvent::Raw(text));
                }
                ParsedLine::CausedBy { .. } | ParsedLine::MoreFrames(_) => {
                    events.push(ParsedEvent::Line(prev_log));
                    events.push(ParsedEvent::Raw(line.to_string()));
                }
            },
            State::AccumulatingStackTrace { log, mut builder } => match parsed {
                ParsedLine::Frame(frame) => {
                    builder.add_frame(frame);
                    self.state = State::AccumulatingStackTrace { log, builder };
                }
                ParsedLine::CausedBy { exception, message } => {
                    builder.add_caused_by(exception, message);
                    self.state = State::AccumulatingStackTrace { log, builder };
                }
                ParsedLine::MoreFrames(_) => {
                    builder.record_more();
                    self.state = State::AccumulatingStackTrace { log, builder };
                }
                ParsedLine::ExceptionHeader { exception, message } => {
                    builder.add_caused_by(exception, message);
                    self.state = State::AccumulatingStackTrace { log, builder };
                }
                ParsedLine::Log(new_log) => {
                    let (trace, lines) = builder.build();
                    events.push(ParsedEvent::ErrorWithTrace {
                        log,
                        trace,
                        line_count: lines,
                    });
                    self.state = State::PendingLog(new_log);
                }
                ParsedLine::Text(text) => {
                    let (trace, lines) = builder.build();
                    events.push(ParsedEvent::ErrorWithTrace {
                        log,
                        trace,
                        line_count: lines,
                    });
                    events.push(ParsedEvent::Raw(text));
                }
            },
        }

        events
    }

    pub fn finish(&mut self) -> Vec<ParsedEvent> {
        let mut events = Vec::new();
        match std::mem::replace(&mut self.state, State::Idle) {
            State::Idle => {}
            State::PendingLog(log) => {
                events.push(ParsedEvent::Line(log));
            }
            State::AccumulatingStackTrace { log, builder } => {
                let (trace, lines) = builder.build();
                events.push(ParsedEvent::ErrorWithTrace {
                    log,
                    trace,
                    line_count: lines,
                });
            }
        }
        events
    }
}
