#![feature(allocator_api)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(prelude_import)]
#![feature(proc_macro_hygiene)]
#![feature(slice_ptr_get)]
#![no_std]

extern crate alloc;
extern crate chips;
extern crate pro_flight;

pub mod tasks;
pub mod thread;

use drone_core::heap;
#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;
use drone_stm32_map::stm32_reg_tokens;
use pro_flight::fake_rtc;

fake_rtc!();

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
