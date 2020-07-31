use core::time::Duration;

use cortex_m::peripheral::{syst::SystClkSource, SYST};

const INTERRUPTS_PER_SECOND: u32 = 100;
const NANOSECONDS_PER_CYCLE: u32 = 1000_000_000 / INTERRUPTS_PER_SECOND;

static mut COUNTER: u32 = 0;

#[exception]
unsafe fn SysTick() {
    core::ptr::write_volatile(&mut COUNTER, COUNTER + 1)
}

// unit microsecond
pub fn get_jiffies() -> Duration {
    let systick = unsafe { &*SYST::ptr() };

    let counter1 = unsafe { core::ptr::read_volatile(&COUNTER) };
    let current = systick.cvr.read();
    let counter = unsafe { core::ptr::read_volatile(&COUNTER) };
    let reload = systick.rvr.read();
    let elapsed = if counter1 != counter { 0 } else { reload - current };
    let secs = (counter / INTERRUPTS_PER_SECOND) as u64;
    let millis = counter % INTERRUPTS_PER_SECOND * 10;
    let nanos = (elapsed as u64 * NANOSECONDS_PER_CYCLE as u64 / reload as u64) as u32;
    Duration::new(secs, millis * 1000_000 + nanos)
}

pub fn systick_init(mut systick: SYST, hz: u32) {
    systick.set_clock_source(SystClkSource::Core);
    systick.set_reload(hz / INTERRUPTS_PER_SECOND - 1);
    systick.enable_counter();
    systick.enable_interrupt();
}
