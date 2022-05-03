use fugit::NanosDurationU64 as Duration;

#[cfg(not(test))]
extern "Rust" {
    fn get_jiffies() -> Duration;
}

#[cfg(test)]
unsafe fn get_jiffies() -> Duration {
    Duration::secs(1)
}

pub fn get() -> Duration {
    unsafe { get_jiffies() }
}
