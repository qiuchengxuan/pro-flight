use core::cmp::min;
use core::fmt::{Result, Write};

pub const INDENT_WIDTH: u16 = 2;

pub struct YamlParser<'a> {
    string: &'a str,
    indent: u16,
    unindent: u16,
}

impl<'a> From<&'a [u8]> for YamlParser<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        let string = unsafe { core::str::from_utf8_unchecked(bytes) };
        Self { string, indent: 0, unindent: 0 }
    }
}

impl<'a> From<&'a str> for YamlParser<'a> {
    fn from(string: &'a str) -> Self {
        Self { string, indent: 0, unindent: 0 }
    }
}

pub fn strip<'a>(bytes: &'a [u8]) -> &'a [u8] {
    if let Some(start) = bytes.iter().position(|&b| b != ' ' as u8) {
        if let Some(end) = bytes.iter().rposition(|&b| b != ' ' as u8) {
            if bytes[start] == '"' as u8 && bytes[end] == '"' as u8 {
                return &bytes[start + 1..end];
            } else {
                return &bytes[start..end + 1];
            }
        }
    }
    bytes
}

impl<'a> YamlParser<'a> {
    fn next_non_blank_line(&mut self) -> Option<&'a str> {
        let mut split = self.string.split('\n');
        while let Some(line) = split.next() {
            let trim_start = line.trim_start();
            if trim_start.is_empty() || trim_start.starts_with("#") {
                self.string = &self.string[min(line.len() + 1, self.string.len())..];
                continue;
            }
            return Some(line);
        }
        None
    }

    fn leave(&mut self) {
        if self.indent == 0 {
            return;
        }
        self.indent -= INDENT_WIDTH;
        while !self.string.is_empty() {
            let line = match self.next_non_blank_line() {
                Some(line) => line,
                None => break,
            };

            let num_leading_space = line.find(|c: char| !c.is_whitespace()).unwrap_or(0);
            if num_leading_space <= self.indent as usize {
                break;
            }
            if self.string.len() != line.len() {
                self.string = &self.string[line.len() + 1..];
            } else {
                self.string = &"";
            }
        }
    }

    fn enter(&mut self, length: usize) {
        self.string = &self.string[length..];
        self.indent += INDENT_WIDTH;
    }

    fn next_indent_matched_line(&mut self) -> Option<&'a str> {
        let line = match self.next_non_blank_line() {
            Some(line) => line,
            None => return None,
        };
        let indent = (self.indent - self.unindent) as usize;
        self.unindent = 0;

        if !line[..indent].trim_start().is_empty() {
            return None;
        }
        self.string = &self.string[indent as usize..];
        Some(&line[indent..])
    }

    pub fn next_entry(&mut self) -> Option<&'a str> {
        if let Some(line) = self.next_indent_matched_line() {
            if let Some(key) = line.splitn(2, ':').next() {
                self.enter(key.len() + 1);
                return Some(key.trim().trim_matches('"'));
            }
        }
        self.leave();
        None
    }

    pub fn next_list_begin(&mut self) -> bool {
        let line = match self.next_non_blank_line() {
            Some(line) => line,
            None => {
                self.leave();
                return false;
            }
        };

        if !line[..self.indent as usize].trim_start().is_empty() {
            self.leave();
            return false;
        }

        if !line[self.indent as usize..].starts_with("- ") {
            self.leave();
            return false;
        }
        self.unindent = self.indent + INDENT_WIDTH;
        self.string = &self.string[(self.indent + INDENT_WIDTH) as usize..];
        self.indent += INDENT_WIDTH;
        true
    }

    pub fn next_value(&mut self) -> Option<&'a str> {
        self.unindent = 0;

        let mut split = self.string.splitn(2, '\n');
        if let Some(line) = split.next() {
            self.string = split.next().unwrap_or_default();
            if !line.trim_start().is_empty() {
                self.leave();
                return Some(line.trim().trim_matches('"'));
            }
        }
        self.leave();
        None
    }

    pub fn next_key_value(&mut self) -> Option<(&'a str, &'a str)> {
        let line = match self.next_indent_matched_line() {
            Some(line) => line,
            None => {
                self.leave();
                return None;
            }
        };
        self.string = &self.string[line.len()..];

        let mut splitter = line.splitn(2, ':');
        if let Some(key) = splitter.next() {
            if let Some(value) = splitter.next() {
                return Some((key.trim().trim_matches('"'), value.trim().trim_matches('"')));
            }
        }
        self.leave();
        None
    }

    pub fn next_list_value(&mut self) -> Option<&'a str> {
        let line = match self.next_indent_matched_line() {
            Some(line) => line,
            None => {
                self.leave();
                return None;
            }
        };
        if line.starts_with("- ") {
            self.string = &self.string[line.len()..];
            return Some((&line[INDENT_WIDTH as usize..]).trim().trim_matches('"'));
        }
        self.leave();
        None
    }

    pub fn skip(&mut self) {
        self.leave()
    }
}

pub trait FromYAML {
    fn from_yaml<'a>(parser: &mut YamlParser<'a>) -> Self;
}

pub trait ToYAML {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result;

    fn write_indent<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        write!(w, "{:indent$}", "", indent = indent * INDENT_WIDTH as usize)
    }
}

mod test {
    #[test]
    fn test_yaml_parser() {
        use super::YamlParser;

        let string = "\
        \ndict:\
        \n  entry-a: a\
        \n  entry-b: b\
        \nmulti-level-dict:\
        \n  level1:\
        \n    level2: lv2\
        \nempty-list: []\
        \nlist:\
        \n  - list-a\
        \n  - list-b\
        \nlist-entry:\
        \n  - entry-a: a\
        \n  - entry-b: b\n";
        let mut stream = YamlParser::from(&string[..]);
        assert_eq!(Some("dict"), stream.next_entry());
        assert_eq!(Some(("entry-a", "a")), stream.next_key_value());
        assert_eq!(Some(("entry-b", "b")), stream.next_key_value());
        assert_eq!(None, stream.next_key_value());
        assert_eq!(Some("multi-level-dict"), stream.next_entry());
        assert_eq!(Some("level1"), stream.next_entry());
        assert_eq!(Some(("level2", "lv2")), stream.next_key_value());
        assert_eq!(None, stream.next_key_value());
        assert_eq!(None, stream.next_entry());
        assert_eq!(Some("empty-list"), stream.next_entry());
        assert_eq!(None, stream.next_list_value());
        assert_eq!(Some("list"), stream.next_entry());
        assert_eq!(Some("list-a"), stream.next_list_value());
        assert_eq!(Some("list-b"), stream.next_list_value());
        assert_eq!(None, stream.next_list_value());
        assert_eq!(Some("list-entry"), stream.next_entry());
        assert_eq!(true, stream.next_list_begin());
        assert_eq!(Some(("entry-a", "a")), stream.next_key_value());
        assert_eq!(None, stream.next_key_value());
        assert_eq!(true, stream.next_list_begin());
        assert_eq!(Some(("entry-b", "b")), stream.next_key_value());
        assert_eq!(None, stream.next_key_value());
        assert_eq!(false, stream.next_list_begin());
    }
}
