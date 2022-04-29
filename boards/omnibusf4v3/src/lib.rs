#![feature(allocator_api)]
#![feature(lang_items)]
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

use core::{alloc::Layout, panic::PanicInfo};

#[cfg(feature = "cortex-m-semihosting")]
use cortex_m_semihosting::hio;
use drivers::stm32::usb_serial;
use drone_core::{heap, heap::Allocator};
use drone_stm32_map::stm32_reg_tokens;
use pro_flight::{
    config::{yaml::YamlParser, Config},
    io::{stdout, Write},
};

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
fn reboot() -> ! {
    cortex_m::peripheral::SCB::sys_reset()
}

#[inline]
fn halt() -> ! {
    cortex_m::asm::bkpt();
    loop {}
}

const DEFAULT_CONFIG: &'static [u8] = core::include_bytes!("../default.config.yaml");

#[no_mangle]
fn default_config() -> Config {
    let config_str = unsafe { core::str::from_utf8_unchecked(DEFAULT_CONFIG) };
    YamlParser::new(config_str).parse()
}

#[no_mangle]
fn stdout_flush() {
    usb_serial::flush()
}

#[no_mangle]
fn stdout_write_bytes(bytes: &[u8]) -> usize {
    #[cfg(feature = "cortex-m-semihosting")]
    match hio::hstdout() {
        Ok(mut stdout) => {
            stdout.write_all(bytes).ok();
        }
        Err(_) => (),
    }
    usb_serial::write_bytes(bytes)
}

#[no_mangle]
fn stdin_read_bytes(buffer: &mut [u8]) -> Result<usize, pro_flight::io::Error> {
    usb_serial::read_bytes(buffer)
}

#[panic_handler]
fn begin_panic(pi: &PanicInfo<'_>) -> ! {
    println!("{}", pi);
    stdout().flush().ok();
    match cfg!(feature = "debug") {
        true => halt(),
        false => reboot(),
    }
}

#[lang = "oom"]
fn oom(layout: Layout) -> ! {
    println!("Couldn't allocate memory of size {}. Aborting!", layout.size());
    stdout().flush().ok();
    match cfg!(feature = "debug") {
        true => halt(),
        false => reboot(),
    }
}
