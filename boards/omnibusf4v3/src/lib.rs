#![feature(allocator_api)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(prelude_import)]
#![feature(proc_macro_hygiene)]
#![feature(slice_ptr_get)]
#![no_std]

extern crate alloc;
#[macro_use]
extern crate chips;
extern crate drivers;
extern crate hal;
#[macro_use]
extern crate log;
#[macro_use]
extern crate pro_flight;

pub mod pwm;
pub mod tasks;
pub mod thread;

use drone_core::heap;
use drone_core::heap::Allocator;
#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;
use drone_stm32_map::stm32_reg_tokens;
use pro_flight::config::{yaml::YamlParser, Config};

mod spi {
    define_spis! {
        Spi1 => (gpioa, PA5, PA6, PA7, AF5, into_alternate_af5)
        Spi2 => (gpiob, PB13, PB14, PB15, AF5, into_alternate_af5)
        Spi3 => (gpioc, PC10, PC11, PC12, AF6, into_alternate_af6)
    }
}

stm32_reg_tokens! {
    /// A set of tokens for all memory-mapped registers.
    index => pub Regs;

    exclude => {
        dwt_cyccnt,
        itm_tpr, itm_tcr, itm_lar,
        tpiu_acpr, tpiu_sppr, tpiu_ffcr,

        scb_ccr,
        mpu_type, mpu_ctrl, mpu_rnr, mpu_rbar, mpu_rasr,
    }
}

heap! {
    config => secondary;
    metadata => pub Heap;
    global => true;
}

/// The global allocator.
#[global_allocator]
pub static HEAP: Heap = Heap::new();

#[no_mangle]
fn heap_statistics() {
    let (mut total, mut free) = (0, 0);
    for pool in HEAP.get_statistics().iter() {
        let (block_size, capacity, remain) = (pool.block_size, pool.capacity, pool.remain);
        println!("Slab {}: capacity {} remain {}", block_size, capacity, remain);
        total += capacity * block_size;
        free += remain * block_size;
    }
    println!("Total: {}, free: {}", total, free);
}

#[no_mangle]
fn board_name() -> &'static str {
    "OMNIBUSF4V3"
}

#[no_mangle]
fn reboot() {
    cortex_m::peripheral::SCB::sys_reset()
}

#[no_mangle]
fn drone_log_is_enabled(_port: u8) -> bool {
    false
}

#[no_mangle]
fn drone_log_flush() {}

#[no_mangle]
fn drone_log_write_bytes(_port: u8, _bytes: &[u8]) {}

#[no_mangle]
fn drone_log_write_u8(port: u8, value: u8) {
    drone_log_write_bytes(port, &value.to_be_bytes())
}

#[no_mangle]
fn drone_log_write_u16(port: u8, value: u16) {
    drone_log_write_bytes(port, &value.to_be_bytes())
}

#[no_mangle]
fn drone_log_write_u32(port: u8, value: u32) {
    drone_log_write_bytes(port, &value.to_be_bytes())
}

const DEFAULT_CONFIG: &'static [u8] = core::include_bytes!("../default.config.yaml");

#[no_mangle]
fn default_config() -> Config {
    let config_str = unsafe { core::str::from_utf8_unchecked(DEFAULT_CONFIG) };
    YamlParser::new(config_str).parse()
}
