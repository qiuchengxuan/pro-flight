use core::fmt::{Result, Write};

pub const INDENT_WIDTH: usize = 2;

pub struct ByteStream<'a> {
    bytes: &'a [u8],
    skip_once_list_mark: bool,
}

impl<'a> From<&'a [u8]> for ByteStream<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self { bytes, skip_once_list_mark: false }
    }
}

#[derive(PartialEq, Debug)]
pub enum Entry<'a> {
    Key(&'a [u8]),
    ListEntry,
    KeyValue(&'a [u8], &'a [u8]),
}

pub fn is_blank(b: u8) -> bool {
    b == ' ' as u8 || b == '\r' as u8 || b == '\n' as u8
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

impl<'a> ByteStream<'a> {
    fn next_non_blank_line(&mut self) -> Option<&'a [u8]> {
        while self.bytes.len() > 0 {
            let mut split = self.bytes.splitn(2, |&b| b == '\n' as u8);
            match split.next() {
                Some(line) => {
                    if !line.iter().all(|&b| is_blank(b)) {
                        return Some(line);
                    }
                    self.bytes = split.next().unwrap_or(&[]);
                }
                None => return None,
            }
        }
        None
    }

    fn next_indent_matched_line(&mut self, indent: usize) -> Option<&'a [u8]> {
        let line = match self.next_non_blank_line() {
            Some(line) => line,
            None => return None,
        };
        if line.len() <= indent || is_blank(line[indent]) {
            return None;
        }
        let num_space = (&line[..indent]).iter().filter(|&&b| b == ' ' as u8).count();
        if num_space == indent {
            return Some(line);
        }
        let has_list_mark = &line[indent - 2..indent] == b"- ";
        if self.skip_once_list_mark && num_space == indent - 1 && has_list_mark {
            self.skip_once_list_mark = false;
            return Some(line);
        }
        None
    }

    pub fn next(&mut self, indent: usize) -> Option<Entry<'a>> {
        let mut line = match self.next_indent_matched_line(indent * INDENT_WIDTH) {
            Some(line) => line,
            None => return None,
        };

        if line[indent * INDENT_WIDTH..].starts_with(b"- ") {
            return Some(Entry::ListEntry);
        }
        if self.bytes.len() != line.len() {
            self.bytes = &self.bytes[line.len() + 1..];
        } else {
            self.bytes = &[];
        }

        line = &line[indent * INDENT_WIDTH..];
        let mut split = line.split(|&b| b == ':' as u8);
        let key = match split.next() {
            Some(key) => strip(key),
            None => return None,
        };

        let value = match split.next() {
            Some(value) => strip(value),
            None => return Some(Entry::Key(key)),
        };
        if value.len() == 0 {
            return Some(Entry::Key(key));
        }
        return Some(Entry::KeyValue(key, value));
    }

    pub fn skip(&mut self, indent: usize) {
        while self.bytes.len() > 0 {
            let line = match self.next_non_blank_line() {
                Some(line) => line,
                None => return,
            };

            let num_leading_space = line.iter().position(|&b| b != ' ' as u8).unwrap_or(0);
            if num_leading_space <= indent * INDENT_WIDTH {
                return;
            }
            if self.bytes.len() != line.len() {
                self.bytes = &self.bytes[line.len() + 1..];
            } else {
                self.bytes = &[];
            }
        }
    }

    pub fn skip_once_list_mark(&mut self) {
        self.skip_once_list_mark = true;
    }
}

pub trait FromYAML {
    fn from_yaml<'a>(&mut self, indent: usize, byte_stream: &mut ByteStream<'a>);
}

pub trait ToYAML {
    fn write_to<W: Write>(&self, indent: usize, w: &mut W) -> Result;

    fn write_indent<W: Write>(&self, indent: usize, w: &mut W) -> Result {
        for _ in 0..indent * INDENT_WIDTH / 2 {
            write!(w, "  ")?
        }
        Ok(())
    }
}

mod test {
    #[test]
    fn test_entry() {
        use super::{ByteStream, Entry};

        let bytes = b"test:\n  a: 0\n";
        let mut stream = ByteStream::from(&bytes[..]);
        let entry = stream.next(0);
        assert_eq!(Some(Entry::Key(b"test")), entry);
        let entry = stream.next(1);
        assert_eq!(Some(Entry::KeyValue(b"a", b"0")), entry);
    }
}
