use core::time;

#[cfg(not(test))]
extern "Rust" {
    fn get_jiffies() -> time::Duration;
}

#[cfg(test)]
unsafe fn get_jiffies() -> time::Duration {
    time::Duration::from_secs(1)
}

pub fn get() -> time::Duration {
    unsafe { get_jiffies() }
}
