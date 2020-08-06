use core::time::Duration;

use cortex_m::peripheral::{syst::SystClkSource, SYST};

const INTERRUPTS_PER_SECOND: u32 = 100;
const NANOSECONDS_PER_CYCLE: u32 = 1000_000_000 / INTERRUPTS_PER_SECOND;

static mut COUNTER: u32 = 0;

#[exception]
unsafe fn SysTick() {
    COUNTER += 1;
}

pub fn get_jiffies() -> Duration {
    let systick = unsafe { &*SYST::ptr() };

    let reload = systick.rvr.read();
    let counter0 = unsafe { core::ptr::read_volatile(&COUNTER) };
    let current0 = systick.cvr.read();
    let counter1 = unsafe { core::ptr::read_volatile(&COUNTER) };
    let current1 = systick.cvr.read();
    let (counter, current) =
        if counter0 != counter1 { (counter1, current1) } else { (counter0, current0) };
    let secs = (counter / INTERRUPTS_PER_SECOND) as u64;
    let millis = counter % INTERRUPTS_PER_SECOND * 10;
    let nanos = ((reload - current) as u64 * NANOSECONDS_PER_CYCLE as u64 / reload as u64) as u32;
    Duration::new(secs, millis * 1000_000 + nanos)
}

pub fn systick_init(mut systick: SYST, hz: u32) {
    systick.set_clock_source(SystClkSource::Core);
    systick.set_reload(hz / INTERRUPTS_PER_SECOND - 1);
    systick.enable_counter();
    systick.enable_interrupt();
}
