#![no_std]
#![feature(llvm_asm)]

#[macro_use]
extern crate cortex_m_rt;
extern crate cortex_m;
extern crate stm32f4xx_hal;

pub mod cortex_m4;
pub mod stm32f4;
