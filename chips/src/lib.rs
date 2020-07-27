#![no_std]
#![feature(llvm_asm)]

#[macro_use]
extern crate cortex_m_rt;
extern crate cortex_m;
#[cfg(feature = "stm32f4xx-hal")]
extern crate stm32f4xx_hal;

pub mod cortex_m4;
#[cfg(feature = "stm32f4xx-hal")]
pub mod stm32f4;
