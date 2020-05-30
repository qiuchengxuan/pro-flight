#![no_std]
#![feature(maybe_uninit_uninit_array)]

#[macro_use]
extern crate enum_map;
extern crate ahrs;
#[macro_use]
extern crate ascii_osd_hud;
extern crate arrayvec;
extern crate ascii;
extern crate nalgebra;
extern crate nb;

pub mod components;
pub mod datastructures;
pub mod drivers;
pub mod hal;

#[cfg(test)]
extern crate std;
