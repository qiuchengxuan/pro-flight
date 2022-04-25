use core::fmt::Write;

use heapless::String;

use super::pathset::{Path, PathSet, Value};

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

    fn parse_sequence<T: PathSet>(&mut self, depth: usize, path_set: &mut T) {
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
                self.parse_map(depth + 1, true, path_set)
            } else {
                let path = Path::new(self.buffer.as_str().split('.'));
                path_set.set(path, Value::of(stripped)).ok();
                self.doc.next();
            }
            index += 1;
            self.buffer.truncate(len)
        }
    }

    fn parse_map<T: PathSet>(&mut self, depth: usize, mut ignore_dash: bool, path_set: &mut T) {
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
                let path = Path::new(self.buffer.as_str().split('.'));
                match value {
                    "" => self.parse_next(depth + 1, path_set),
                    "[]" | "~" | "null" => {
                        path_set.set(path, Value(None)).ok();
                    }
                    _ => {
                        path_set.set(path, Value::of(value)).ok();
                    }
                }
            }
            self.buffer.truncate(len);
        }
    }

    fn parse_next<T: PathSet>(&mut self, depth: usize, path_set: &mut T) {
        if !self.next_line(depth, false) {
            return;
        }

        let line = &self.doc.clone().next().unwrap()[depth * self.indent_width..];
        if line.starts_with("- ") {
            return self.parse_sequence(depth, path_set);
        }

        if line.contains(':') {
            return self.parse_map(depth, false, path_set);
        }
    }

    pub fn parse_into<T: PathSet>(&mut self, path_set: &mut T) {
        self.parse_next(0, path_set);
    }

    pub fn parse<T: Default + PathSet>(&mut self) -> T {
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
        use std::fmt::Write;

        use super::YamlParser;
        use crate::config::pathset::{Error, Path, Value};

        struct Handler(pub String);

        impl super::PathSet for Handler {
            fn set(&mut self, path: Path, value: Value) -> Result<(), Error> {
                let split = path.unwrap();
                if let Some(v) = value.0 {
                    writeln!(self.0, "{} = {}", split.collect::<Vec<&str>>().join("."), v).ok();
                } else {
                    writeln!(self.0, "{} = null", split.collect::<Vec<&str>>().join(".")).ok();
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
