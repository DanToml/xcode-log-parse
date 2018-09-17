use common::{ParserEvent, ParserResult};
use std::path::Path;
use std::collections::HashMap;
use std::fmt;

pub trait Formatter {
    fn format(&mut self, r: ParserResult);
}

pub struct PlainTextFormatter {
    level: usize,
}

impl PlainTextFormatter {
    pub fn new() -> PlainTextFormatter {
        PlainTextFormatter {
            level: 0,
        }
    }

    fn print_event(&mut self, event: ParserEvent) {
        match event {
            ParserEvent::Message(message_type, contents) => {
                let type_emoji = match message_type.as_ref() {
                    "note" => "📝 " ,
                    "warning" => "⚠️ ",
                    "error" => "❌ ",
                    _ => "",
                };
                println!("{}{}{}: {}", self.indentation(), type_emoji, message_type, contents);
            },
            ParserEvent::Status(name, outcome, Some(duration)) => println!("{}{} {} [{}]\n", self.indentation(), name, outcome, duration),
            ParserEvent::Status(name, outcome, None) => println!("{}{} {}\n", self.indentation(), name, outcome),
            ParserEvent::BeginCommand(name, args) => {
                let pretty_name = match name.as_ref() {
                    "CompileC" | "CompileXIB" => "Compiling",
                    "CpResource" | "CopyStringsFile" => "Copying",
                    "ProcessPCH" => "Precompiling",
                    "Ld" => "Linking",
                    _ => return,
                };
                let first_arg = args.first().unwrap();
                let first_arg = Path::new(first_arg).file_name().unwrap().to_str().unwrap();
                self.level += 1;
                println!("{}▸ {} {}", self.indentation(), pretty_name, first_arg);
            },
            ParserEvent::EndCommand(_, _) => self.level -= 1,
            ParserEvent::BeginSubCommand(_, _) => self.level += 1,
            ParserEvent::EndSubCommand(_, _) => self.level -= 1,
            ParserEvent::BeginTarget(name) => println!("🛠  Building {}", name),
        }
    }

    fn indentation(&self) -> String {
        "  ".repeat(self.level - self.level)
    }
}

impl Formatter for PlainTextFormatter {
    fn format(&mut self, r: ParserResult) {
        match r {
            ParserResult::Commands(names) => {
                for name in names {
                    self.print_event(name);
                }
            }
            ParserResult::Continue => {},
            ParserResult::NoMatch => println!("NoMatch"),
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
enum Counted {
    Target,
    Message(String),
    Command(String),
}
impl fmt::Display for Counted {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Counted::Command(x) => write!(f, "{}", x),
            Counted::Message(t) => write!(f, "{}", t),
            _ => write!(f, "{:?}", *self),
        }
    }
}

#[derive(Default)]
pub struct LiveCounterFormatter {
    counts: HashMap<Counted, u32>,
    lines_printed: usize,
}
impl LiveCounterFormatter {
    pub fn new() -> LiveCounterFormatter {
        LiveCounterFormatter::default()
    }

    fn update_for_event(&mut self, event: ParserEvent) {
        match event {
            ParserEvent::BeginTarget(_) => self.increment(Counted::Target),
            ParserEvent::Message(t, _) => self.increment(Counted::Message(t)),
            ParserEvent::BeginCommand(x, _) => self.increment(Counted::Command(x)),
            _ => {},
        }
    }

    fn increment(&mut self, key: Counted) {
        let value = self.counts.get(&key).unwrap_or(&0) + 1;
        self.counts.insert(key, value);
    }

    fn print(&mut self) {
        if self.lines_printed > 0 {
            print!("{}[{}A", 27 as char, self.lines_printed);
        }
        for (key, value) in self.counts.iter() {
            println!("{}[0K{}: {}", 27 as char, key, value);
        }
        self.lines_printed = self.counts.len();
    }
}
impl Formatter for LiveCounterFormatter {
    fn format(&mut self, r: ParserResult) {
        match r {
            ParserResult::Commands(names) => {
                for name in names {
                    self.update_for_event(name);
                }
                self.print();
            }
            ParserResult::Continue => {},
            ParserResult::NoMatch => println!("NoMatch"),
        }
    }
}
