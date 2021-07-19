use core::time::Duration;

const NANOS_PER_SEC: u32 = 1_000_000_000;
const NANOS_PER_MILLI: u32 = 1_000_000;
const MILLIS_PER_SEC: u32 = 1_000;

#[cfg(not(test))]
extern "Rust" {
    fn get_jiffies() -> u64; // nano seconds
}

#[cfg(test)]
unsafe fn get_jiffies() -> u64 {
    0
}

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Jiffies(u64);

impl Jiffies {
    pub fn as_secs(&self) -> u64 {
        self.0 / NANOS_PER_SEC as u64
    }

    pub fn subsec_millis(&self) -> u32 {
        self.0 as u32 / NANOS_PER_MILLI % MILLIS_PER_SEC
    }

    pub fn subsec_nanos(&self) -> u32 {
        self.0 as u32 % NANOS_PER_SEC
    }
}

impl core::ops::Add for Jiffies {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl From<Duration> for Jiffies {
    fn from(duration: Duration) -> Self {
        Self(duration.as_secs() * NANOS_PER_SEC as u64 + duration.subsec_nanos() as u64)
    }
}

pub fn get() -> Jiffies {
    Jiffies(unsafe { get_jiffies() })
}
