use core::{
    fmt::{self, Display, Formatter, Write},
    str::from_utf8_unchecked,
    sync::atomic::{AtomicUsize, Ordering},
};

use log::{Log, Metadata, Record};

use crate::sys::jiffies;

#[derive(Default)]
pub struct LogBuffer {
    buffer: &'static mut [u8],
    index: AtomicUsize,
    writer_count: AtomicUsize,
}

impl Write for LogBuffer {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let size = self.buffer.len();
        if size == 0 {
            return Ok(());
        }
        self.writer_count.fetch_add(1, Ordering::Relaxed);
        let index = self.index.fetch_add(1, Ordering::Relaxed);
        let mut buf = [0u8; 2];
        let bytes = c.encode_utf8(&mut buf).as_bytes();
        self.buffer[index % size] = bytes[0];
        if bytes.len() > 1 {
            self.buffer[(index + 1) % size] = bytes[1];
        }
        self.writer_count.fetch_sub(1, Ordering::Release);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.writer_count.fetch_add(1, Ordering::Relaxed);
        let mut bytes = s.as_bytes();
        if self.buffer.len() <= bytes.len() {
            bytes = &bytes[..self.buffer.len()];
        }
        let size = self.buffer.len();
        let index = self.index.fetch_add(bytes.len(), Ordering::Relaxed) % size;

        if size - index > bytes.len() {
            self.buffer[index..index + bytes.len()].copy_from_slice(bytes);
            self.writer_count.fetch_sub(1, Ordering::Release);
            return Ok(());
        }

        let partial_size = size - index;
        self.buffer[index..size].copy_from_slice(&bytes[..partial_size]);
        self.buffer[..bytes.len() - partial_size].copy_from_slice(&bytes[partial_size..]);
        self.writer_count.fetch_sub(1, Ordering::Release);
        Ok(())
    }
}

impl Display for LogBuffer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        while self.writer_count.load(Ordering::Relaxed) > 0 {}
        core::sync::atomic::fence(Ordering::Acquire);
        let index = self.index.load(Ordering::Relaxed);
        if index <= self.buffer.len() {
            return write!(f, "{}", unsafe { from_utf8_unchecked(&self.buffer[..index]) });
        }
        let index = index % self.buffer.len();
        write!(f, "{}", unsafe { from_utf8_unchecked(&self.buffer[index..]) })?;
        write!(f, "{}", unsafe { from_utf8_unchecked(&self.buffer[..index]) })
    }
}

static mut LOG_BUFFER: LogBuffer =
    LogBuffer { buffer: &mut [], index: AtomicUsize::new(0), writer_count: AtomicUsize::new(0) };

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let jiffies = jiffies::get();
        let millis = jiffies.to_millis() as u32;
        println!("[{:5}.{:03}] {}", millis / 1000, millis % 1000, record.args());
        let log_buffer = unsafe { &mut LOG_BUFFER };
        writeln!(log_buffer, "[{:5}.{:03}] {}", millis / 1000, millis % 1000, record.args()).ok();
    }

    fn flush(&self) {}
}

pub fn get() -> &'static LogBuffer {
    unsafe { &LOG_BUFFER }
}

pub fn init(buffer: &'static mut [u8]) {
    unsafe { LOG_BUFFER = LogBuffer { buffer, ..Default::default() } }
    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(&Logger).ok();
}
