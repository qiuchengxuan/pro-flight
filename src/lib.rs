#![no_std]
#![feature(maybe_uninit_uninit_array)]

#[macro_use]
extern crate enum_map;
extern crate ascii_osd_hud;
extern crate nalgebra;

pub mod components;
pub mod datastructures;
pub mod drivers;
pub mod hal;

#[cfg(test)]
#[macro_use]
extern crate std;
