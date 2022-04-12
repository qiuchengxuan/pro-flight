use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use drone_core::{fib::Yielded, thr::ThrToken};
use drone_cortexm::{reg::prelude::*, thr::prelude::*};
use drone_stm32_map::periph::sys_tick::SysTickPeriph;

use super::clock::SYSCLK;

pub const RATE: u32 = 1000;
static mut SYS_TICK_PERIPH: MaybeUninit<SysTickPeriph> = MaybeUninit::uninit();
static COUNTER: AtomicU32 = AtomicU32::new(0);

#[no_mangle]
fn get_jiffies() -> Duration {
    let systick = unsafe { &mut *SYS_TICK_PERIPH.as_mut_ptr() };
    let counter = COUNTER.load(Ordering::Acquire);
    let val = SYSCLK / RATE - 1 - systick.stk_val.current.read_bits();
    let counter2 = COUNTER.load(Ordering::Relaxed);
    if counter < counter2 {
        return Duration::from_millis(counter2 as u64);
    }
    Duration::from_nanos(counter as u64 * 1000_000 + (val * 1000 / (SYSCLK / RATE / 1000)) as u64)
}

pub fn init(systick: SysTickPeriph, thread: impl ThrToken) {
    systick.stk_val.store(|r| r.write_current(0));
    systick.stk_load.store(|r| r.write_reload(SYSCLK / RATE - 1));
    systick.stk_ctrl.store(|r| r.set_clksource().set_tickint().set_enable());
    unsafe { SYS_TICK_PERIPH = MaybeUninit::new(systick) };
    thread.add_fn(move || {
        COUNTER.fetch_add(1, Ordering::Release);
        Yielded::<(), ()>(())
    });
}
