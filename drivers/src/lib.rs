#![no_std]

extern crate alloc;
extern crate hal;
#[macro_use]
extern crate mpu6000 as _;
#[macro_use]
extern crate pro_flight;

pub mod led;
pub mod mpu6000;
pub mod nvram;
