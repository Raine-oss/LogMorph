// Log Stream Reader

use crate::parser::state_machine::{ParsedEvent, StateMachineParser};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

// Stream Processor

pub struct LogStreamReader<R: BufRead> {
    reader: R,
    parser: StateMachineParser,
    buffer: String,
    event_queue: Vec<ParsedEvent>,
    finished: bool,
}

impl<R: BufRead> LogStreamReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            parser: StateMachineParser::new(),
            buffer: String::with_capacity(512),
            event_queue: Vec::new(),
            finished: false,
        }
    }
}

impl LogStreamReader<BufReader<File>> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self::new(reader))
    }
}

impl LogStreamReader<BufReader<io::Stdin>> {
    pub fn from_stdin() -> Self {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        Self::new(reader)
    }
}

// Iterator Implementation

impl<R: BufRead> Iterator for LogStreamReader<R> {
    type Item = io::Result<ParsedEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.event_queue.is_empty() {
                return Some(Ok(self.event_queue.remove(0)));
            }

            if self.finished {
                return None;
            }

            self.buffer.clear();
            match self.reader.read_line(&mut self.buffer) {
                Ok(0) => {
                    self.finished = true;
                    let trailing = self.parser.finish();
                    if trailing.is_empty() {
                        return None;
                    }
                    self.event_queue.extend(trailing);
                }
                Ok(_) => {
                    let line = self.buffer.trim_end_matches(&['\r', '\n'][..]);
                    let events = self.parser.process_line(line);
                    if !events.is_empty() {
                        self.event_queue.extend(events);
                    }
                }
                Err(e) => {
                    self.finished = true;
                    return Some(Err(e));
                }
            }
        }
    }
}
