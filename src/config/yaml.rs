use core::fmt::Write;

use heapless::String;

use super::setter::{Setter, Value};

pub struct YamlParser<'a> {
    doc: core::str::Lines<'a>,
    indent_width: usize,
    buffer: String<80>,
}

impl<'a> YamlParser<'a> {
    fn skip_blank_lines(&mut self) {
        let lines = self.doc.clone();
        for line in lines {
            if line.find(|c: char| !c.is_ascii_whitespace()).is_some() {
                break;
            }
            self.doc.next();
        }
    }

    fn next_line(&mut self, depth: usize, ignore_dash: bool) -> bool {
        self.skip_blank_lines();
        let lines = self.doc.clone();
        for line in lines {
            let index = match line.find(|c| !(c == ' ' || (ignore_dash && c == '-'))) {
                Some(index) => index,
                None => return false,
            };
            if index == depth * self.indent_width {
                return true;
            } else if index < depth * self.indent_width {
                return false;
            }
            self.doc.next();
        }
        false
    }

    fn parse_sequence(&mut self, depth: usize, setter: &mut dyn Setter) {
        let len = self.buffer.len();
        let mut index = 0;
        while self.next_line(depth, false) {
            let line = match self.doc.clone().next() {
                Some(line) => line,
                None => break,
            };
            let mut stripped = &line[depth * self.indent_width..];
            if !stripped.starts_with("- ") {
                continue;
            }
            write!(self.buffer, "[{}]", index).ok();
            stripped = (&stripped[2..]).trim_end();
            if stripped.contains(':') {
                self.parse_map(depth + 1, true, setter)
            } else {
                setter.set(&mut self.buffer.as_str().split('.'), Value::of(stripped)).ok();
                self.doc.next();
            }
            index += 1;
            self.buffer.truncate(len)
        }
    }

    fn parse_map(&mut self, depth: usize, mut ignore_dash: bool, setter: &mut dyn Setter) {
        let len = self.buffer.len();
        while self.next_line(depth, ignore_dash) {
            ignore_dash = false;
            let mut line = match self.doc.next() {
                Some(line) => line,
                None => break,
            };
            line = &line[depth * self.indent_width..];
            let mut splitted = line.splitn(2, ':');
            let key = match splitted.next() {
                Some(key) => key,
                None => continue,
            };
            if depth > 0 {
                self.buffer.push_str(".").ok();
            }
            self.buffer.push_str(key).ok();

            let trim = ['\'', '"', ' '];
            if let Some(value) = splitted.next().map(|v| v.trim_matches(&trim[..])) {
                match value {
                    "" => self.parse_next(depth + 1, setter),
                    "[]" | "~" | "null" => {
                        setter.set(&mut self.buffer.as_str().split('.'), Value(None)).ok();
                    }
                    _ => {
                        setter.set(&mut self.buffer.as_str().split('.'), Value::of(value)).ok();
                    }
                }
            }
            self.buffer.truncate(len);
        }
    }

    fn parse_next(&mut self, depth: usize, setter: &mut dyn Setter) {
        if !self.next_line(depth, false) {
            return;
        }

        let line = &self.doc.clone().next().unwrap()[depth * self.indent_width..];
        if line.starts_with("- ") {
            return self.parse_sequence(depth, setter);
        }

        if line.contains(':') {
            return self.parse_map(depth, false, setter);
        }
    }

    pub fn parse_into(&mut self, setter: &mut dyn Setter) {
        self.parse_next(0, setter);
    }

    pub fn parse<T: Default + Setter>(&mut self) -> T {
        let mut value = T::default();
        self.parse_into(&mut value);
        value
    }

    pub fn new(doc: &'a str) -> Self {
        Self { doc: doc.lines(), indent_width: 2, buffer: String::new() }
    }
}

mod test {
    #[test]
    fn test_yaml_parser() {
        use core::str::Split;
        use std::fmt::Write;

        use super::YamlParser;
        use crate::config::setter::{Error, Value};

        struct Handler(pub String);

        impl super::Setter for Handler {
            fn set(&mut self, path: &mut Split<char>, value: Value) -> Result<(), Error> {
                if let Some(v) = value.0 {
                    writeln!(self.0, "{} = {}", path.collect::<Vec<&str>>().join("."), v).ok();
                } else {
                    writeln!(self.0, "{} = null", path.collect::<Vec<&str>>().join(".")).ok();
                }
                Ok(())
            }
        }

        let mut handler = Handler(String::new());

        let string = r#"
        dict:
          entry-a: a
          entry-b: b
        multi-level-dict:
          level1:
            level2: lv2
        empty-list: []
        list:
          - list-a
          - list-b
        list-entry:
          - entry-a: a
          - entry-b: b"#;
        YamlParser::new(&string.replace("        ", "")).parse_into(&mut handler);

        let expected = r#"
        dict.entry-a = a
        dict.entry-b = b
        multi-level-dict.level1.level2 = lv2
        empty-list = null
        list[0] = list-a
        list[1] = list-b
        list-entry[0].entry-a = a
        list-entry[1].entry-b = b"#;
        assert_eq!(expected.replace("        ", "").trim(), handler.0.trim());
    }
}
