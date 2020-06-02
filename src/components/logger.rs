use core::fmt;
use core::sync::atomic::{AtomicPtr, Ordering};

static mut LOG_BUFFER: &'static mut [u8] = &mut [0u8; 0];
static mut INDEX: AtomicPtr<usize> = AtomicPtr::new(0 as *mut usize);

pub struct Logger;

impl Logger {
    fn allocate(&self, len: usize) -> usize {
        let mut index = unsafe { INDEX.load(Ordering::Relaxed) };
        loop {
            let new_index = (index as usize + len) as *mut usize;
            let current = unsafe { INDEX.compare_and_swap(index, new_index, Ordering::Relaxed) };
            if current == index {
                return index as usize;
            }
            index = current;
        }
    }
}

impl fmt::Write for Logger {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let buffer = unsafe { &mut LOG_BUFFER };
        if buffer.len() == 0 {
            return Ok(());
        }
        let index = self.allocate(1);
        buffer[index % buffer.len()] = c as u8;
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        let buffer = unsafe { &mut LOG_BUFFER };
        let bytes = s.as_bytes();
        if buffer.len() <= bytes.len() {
            return Ok(());
        }
        let index = self.allocate(bytes.len() + 2) % buffer.len();
        if index + bytes.len() < buffer.len() {
            buffer[index..index + bytes.len()].copy_from_slice(bytes);
        } else {
            let size = buffer.len() - index;
            buffer[index..index + size].copy_from_slice(&bytes[..size]);
            buffer[..bytes.len() - size].copy_from_slice(&bytes[size..]);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        write!(&mut Logger{}, $($arg)*).ok();
        write!(&mut Logger{}, "\r\n").ok()
    };
}

pub fn init(buffer: &'static mut [u8]) {
    unsafe { LOG_BUFFER = buffer }
}

pub struct LogReader((usize, usize));

impl Iterator for LogReader {
    type Item = &'static [u8];

    fn next(&mut self) -> Option<&'static [u8]> {
        let (index, count) = self.0;
        let log_size = unsafe { LOG_BUFFER.len() };
        if index <= log_size {
            if count == 0 {
                self.0 = (index, count + 1);
                return Some(unsafe { &LOG_BUFFER[..index] });
            } else {
                return None;
            }
        }
        if count == 0 {
            self.0 = (index, count + 1);
            return Some(unsafe { &LOG_BUFFER[index..] });
        } else if count == 1 {
            self.0 = (index, count + 1);
            return Some(unsafe { &LOG_BUFFER[..index] });
        }
        None
    }
}

pub fn reader() -> LogReader {
    let index = unsafe { INDEX.load(Ordering::Relaxed) as usize };
    LogReader((index, 0))
}

mod test {
    #[test]
    fn write_log() {
        use super::Logger;
        use core::fmt::Write;

        static mut BUFFER: [u8; 100] = [0u8; 100];
        super::init(unsafe { &mut BUFFER });
        log!("test a");
        log!("test b");
        assert_eq!(super::reader().next().unwrap(), b"test a\r\ntest b\r\n");
    }
}
