#![no_std]

extern crate alloc;
extern crate bmp280_core as bmp280;
extern crate hal;
#[macro_use]
extern crate mpu6000 as _;
#[macro_use]
extern crate pro_flight;

pub mod barometer;
pub mod led;
pub mod mpu6000;
pub mod nvram;
#[cfg(feature = "stm32")]
pub mod stm32;
