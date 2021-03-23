#![cfg_attr(not(test), no_std)]
#![feature(trait_alias)]

extern crate alloc;
extern crate ascii;
#[cfg(not(test))]
#[macro_use]
extern crate drone_core;
extern crate heapless;
extern crate integer_sqrt;
extern crate micromath;
#[macro_use]
extern crate mpu6000;
extern crate nalgebra;
extern crate nb;
#[macro_use]
extern crate sval;
extern crate sval_json;
extern crate usb_device;
extern crate usbd_serial;

#[macro_use]
pub mod sys;
pub mod components;
#[macro_use]
pub mod datastructures;
pub mod config;
pub mod drivers;
pub mod hal;
#[cfg(test)]
extern crate std;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(test)]
#[macro_use]
extern crate serial_test;
