use core::sync::atomic::{AtomicBool, Ordering};

pub trait Read {
    type Error;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

pub trait Write {
    type Error;
    fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error>;
}

pub enum Error {
    Locked,
}

extern "Rust" {
    fn stdin_read_bytes(buffer: &mut [u8]) -> Result<usize, Error>;
}

static STDIN_LOCK: AtomicBool = AtomicBool::new(false);

pub struct Stdin(bool);

pub fn stdin() -> Stdin {
    Stdin(false)
}

impl Stdin {
    pub fn lock(&mut self) -> bool {
        if STDIN_LOCK
            .compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            self.0 = true;
        }
        self.0
    }

    pub fn unlock(&mut self) {
        if self.0 {
            STDIN_LOCK.store(false, Ordering::Relaxed);
            self.0 = false;
        }
    }
}

impl Drop for Stdin {
    fn drop(&mut self) {
        self.unlock()
    }
}

impl Read for Stdin {
    type Error = Error;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if self.0 {
            return unsafe { stdin_read_bytes(buf) };
        }
        if !self.lock() {
            return Err(Error::Locked);
        }
        let result = unsafe { stdin_read_bytes(buf) };
        self.unlock();
        result
    }
}
