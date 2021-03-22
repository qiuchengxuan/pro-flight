//! The root task.

use chips::stm32f4::{clock, systick};
use drone_core::fib::{new_fn, ThrFiberStreamPulse, Yielded};
use drone_cortexm::{reg::prelude::*, thr::prelude::*};
use drone_stm32_map::periph::sys_tick::periph_sys_tick;
use futures::prelude::*;
use pro_flight::{drivers::led::LED, sys::timer};
use stm32f4xx_hal::{prelude::*, stm32};

use crate::{thread, thread::ThrsInit, Regs};

/// The root task handler.
#[inline(never)]
pub fn handler(reg: Regs, thr_init: ThrsInit) {
    let mut thread = thread::init(thr_init);
    thread.hard_fault.add_once(|| panic!("Hard Fault"));
    thread.rcc.enable_int();
    let rcc_cir = reg.rcc_cir.into_copy();

    reg.rcc_ahb1enr.modify(|r| r.set_dma2en());
    reg.rcc_apb1enr.pwren.set_bit();
    reg.rcc_apb2enr.modify(|r| r.set_spi1en());

    let regs = (reg.rcc_cfgr, reg.rcc_cr, reg.rcc_pllcfgr);
    clock::setup_pll(&mut thread.rcc, rcc_cir, regs, &reg.flash_acr).root_wait();
    systick::init(periph_sys_tick!(reg), thread.sys_tick);

    let peripherals = stm32::Peripherals::take().unwrap();
    let gpio_b = peripherals.GPIOB.split();
    let mut led = LED::new(gpio_b.pb5.into_push_pull_output(), timer::SysTimer::new());

    reg.pwr_cr.modify(|r| r.set_dbp());
    reg.rcc_bdcr.modify(|r| r.set_rtcsel1().set_rtcsel0().set_rtcen()); // select HSE

    let mut stream = thread.sys_tick.add_saturating_pulse_stream(new_fn(move || Yielded(Some(1))));
    while let Some(_) = stream.next().root_wait() {
        led.check_toggle();
    }

    reg.scb_scr.sleeponexit.set_bit(); // Enter a sleep state on ISR exit.
}
