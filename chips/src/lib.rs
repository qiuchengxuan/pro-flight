#![no_std]
#![feature(asm)]

extern crate alloc;
#[cfg(feature = "stm32")]
extern crate cortex_m;
#[cfg(feature = "stm32")]
extern crate drone_cortexm;
#[cfg(feature = "stm32")]
extern crate drone_stm32_map;
#[cfg(feature = "stm32")]
extern crate stm32f4xx_hal;

#[cfg(feature = "stm32")]
pub mod stm32f4;
