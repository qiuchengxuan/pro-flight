use core::fmt::Write;

pub struct ByteIter<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for ByteIter<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }
}

#[derive(PartialEq, Debug)]
pub enum Entry<'a> {
    Key(&'a [u8]),
    ListEntry,
    KeyValue(&'a [u8], &'a [u8]),
    None,
}

pub fn is_blank(b: u8) -> bool {
    b == ' ' as u8 || b == '\r' as u8 || b == '\n' as u8
}

pub fn strip<'a>(bytes: &'a [u8]) -> &'a [u8] {
    let mut size = bytes.len();
    for i in 1..bytes.len() + 1 {
        if !is_blank(bytes[bytes.len() - i]) {
            size = bytes.len() - i + 1;
            break;
        }
    }
    let bytes = &bytes[..size];
    if bytes.len() >= 2 && bytes[0] == '"' as u8 {
        return &bytes[0..bytes.len() - 1];
    }
    bytes
}

#[inline]
pub fn num_leading_space(bytes: &[u8]) -> usize {
    match bytes.iter().position(|&b| b != ' ' as u8) {
        Some(index) => index,
        None => 0,
    }
}

#[inline]
pub fn is_blank_line(bytes: &[u8]) -> bool {
    bytes.iter().all(|&b| is_blank(b))
}

impl<'a> ByteIter<'a> {
    #[inline]
    fn next_line(&self) -> &'a [u8] {
        let index = self.0.iter().position(|&b| b == '\n' as u8).unwrap_or(self.0.len() - 1);
        &self.0[..index + 1]
    }

    pub fn next_non_blank_line(&mut self) -> Option<(&'a [u8], usize)> {
        let mut line = self.next_line();
        let mut num_space = num_leading_space(line);
        while is_blank_line(&line[num_space..]) {
            if line.len() == 0 {
                return None;
            }
            self.0 = &self.0[line.len()..];
            line = self.next_line();
            num_space = num_leading_space(line);
        }
        Some((line, num_space))
    }

    pub fn next(&mut self, indent: usize) -> Entry<'a> {
        if self.0.len() == 0 {
            return Entry::None;
        }

        let (mut line, num_space) = match self.next_non_blank_line() {
            Some(tuple) => tuple,
            None => return Entry::None,
        };

        if num_space != indent {
            return Entry::None;
        }

        if line.len() > 2 && &line[..2] == b"- " {
            self.0 = &self.0[num_space + 2..];
            return Entry::ListEntry;
        }
        self.0 = &self.0[line.len()..];

        line = &line[num_space..];
        let index = match line.iter().position(|&b| b == ':' as u8) {
            Some(i) => i,
            None => return Entry::None,
        };
        let key = strip(&line[..index]);
        line = &line[index + 1..];

        let next_non_blank = match line.iter().position(|&b| !is_blank(b)) {
            Some(index) => index,
            None => return Entry::Key(key),
        };

        let value = strip(&line[next_non_blank..]);
        if value.len() == 0 {
            return Entry::Key(key);
        }
        return Entry::KeyValue(key, value);
    }

    pub fn skip(&mut self, indent: usize) {
        while self.0.len() > 0 {
            let (line, num_space) = match self.next_non_blank_line() {
                Some(tuple) => tuple,
                None => return,
            };
            if num_space <= indent {
                return;
            }
            self.0 = &self.0[line.len()..];
        }
    }
}

pub trait FromYAML {
    fn from_yaml<'a>(&mut self, indent: usize, byte_iter: &mut ByteIter<'a>);
}

pub trait ToYAML {
    fn to_writer<W: Write>(self, w: W);
}

mod test {
    #[test]
    fn test_entry() {
        use super::{ByteIter, Entry};

        let bytes = b"test:\n  a: 0\n";
        let mut iter = ByteIter::from(&bytes[..]);
        let entry = iter.next(0);
        assert_eq!(Entry::Key(b"test"), entry);
        let entry = iter.next(2);
        assert_eq!(Entry::KeyValue(b"a", b"0"), entry);
    }
}
