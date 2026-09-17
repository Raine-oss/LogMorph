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
    more_count: usize,
    lines_consumed: usize,
}

impl TraceBuilder {
    fn new(exception: String, message: Option<String>, initial_lines: usize) -> Self {
        Self {
            exception,
            message,
            frames: Vec::new(),
            caused_by: None,
            more_count: 0,
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

    fn record_more(&mut self, count: usize) {
        self.lines_consumed += 1;
        if let Some(ref mut child) = self.caused_by {
            child.record_more(count);
        } else {
            self.more_count = count;
        }
    }

    fn build(mut self, parent_frames: Option<&[StackFrame]>) -> (StackTraceBlock, usize) {
        let lines = self.lines_consumed;

        if let Some(parents) = parent_frames {
            if self.more_count > 0 && !parents.is_empty() {
                let p_len = parents.len();
                let start_idx = p_len.saturating_sub(self.more_count);
                let candidate_slice = &parents[start_idx..p_len];

                for candidate in candidate_slice {
                    let already_present = self.frames.iter().any(|existing| {
                        existing.class_name == candidate.class_name
                            && existing.method_name == candidate.method_name
                            && existing.line_number == candidate.line_number
                    });

                    if !already_present {
                        self.frames.push(candidate.clone());
                    }
                }
            }
        }

        let current_frames = self.frames.clone();
        let built_cause = self.caused_by.map(|c| Box::new(c.build(Some(&current_frames)).0));

        let block = StackTraceBlock {
            primary_exception: self.exception,
            exception_message: self.message,
            frames: self.frames,
            caused_by: built_cause,
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
                ParsedLine::MoreFrames(count) => {
                    builder.record_more(count);
                    self.state = State::AccumulatingStackTrace { log, builder };
                }
                ParsedLine::ExceptionHeader { exception, message } => {
                    builder.add_caused_by(exception, message);
                    self.state = State::AccumulatingStackTrace { log, builder };
                }
                ParsedLine::Log(new_log) => {
                    let (trace, lines) = builder.build(None);
                    events.push(ParsedEvent::ErrorWithTrace {
                        log,
                        trace,
                        line_count: lines,
                    });
                    self.state = State::PendingLog(new_log);
                }
                ParsedLine::Text(text) => {
                    let (trace, lines) = builder.build(None);
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
                let (trace, lines) = builder.build(None);
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

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_simple_log_flow() {
        let mut sm = StateMachineParser::new();
        let evs1 = sm.process_line("[10:00:00 INFO]: First line");
        assert!(evs1.is_empty());

        let evs2 = sm.process_line("[10:00:01 INFO]: Second line");
        assert_eq!(evs2.len(), 1);
        match &evs2[0] {
            ParsedEvent::Line(l) => assert_eq!(l.message, "First line"),
            _ => panic!("Expected Line"),
        }

        let evs3 = sm.finish();
        assert_eq!(evs3.len(), 1);
        match &evs3[0] {
            ParsedEvent::Line(l) => assert_eq!(l.message, "Second line"),
            _ => panic!("Expected Line"),
        }
    }

    #[test]
    fn test_state_machine_error_with_trace() {
        let mut sm = StateMachineParser::new();
        sm.process_line("[10:00:00 ERROR]: Something broke");
        sm.process_line("java.lang.NullPointerException: Object is null");
        sm.process_line("\tat com.example.Test.run(Test.java:15)");
        sm.process_line("Caused by: java.lang.IllegalArgumentException: Bad arg");
        sm.process_line("\tat com.example.Test.init(Test.java:5)");
        sm.process_line("[10:00:01 INFO]: Server started");

        let finished = sm.finish();
        assert_eq!(finished.len(), 1);
    }

    #[test]
    fn test_reconstruct_more_frames_deduplication() {
        let mut sm = StateMachineParser::new();
        sm.process_line("[10:00:00 ERROR]: Failed");
        sm.process_line("java.lang.RuntimeException: Root error");
        sm.process_line("\tat com.example.Top.call(Top.java:10)");
        sm.process_line("\tat com.example.Middle.call(Middle.java:20)");
        sm.process_line("\tat com.example.Bottom.run(Bottom.java:30)");
        sm.process_line("Caused by: java.io.IOException: Sub error");
        sm.process_line("\tat com.example.Sub.action(Sub.java:5)");
        sm.process_line("\tat com.example.Middle.call(Middle.java:20)");
        sm.process_line("\t... 2 more");

        let evs = sm.finish();
        assert_eq!(evs.len(), 1);
        match &evs[0] {
            ParsedEvent::ErrorWithTrace { trace, .. } => {
                let cause = trace.caused_by.as_ref().expect("Expected cause");
                assert_eq!(cause.frames.len(), 3);
                assert_eq!(cause.frames[0].class_name, "com.example.Sub");
                assert_eq!(cause.frames[1].class_name, "com.example.Middle");
                assert_eq!(cause.frames[2].class_name, "com.example.Bottom");
            }
            _ => panic!("Expected ErrorWithTrace"),
        }
    }
}
