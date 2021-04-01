#![cfg_attr(not(test), no_std)]
#![feature(trait_alias)]

#[macro_use]
extern crate alloc;
extern crate ascii;
#[cfg(not(test))]
#[macro_use]
extern crate drone_core;
extern crate heapless;
extern crate integer_sqrt;
extern crate micromath;
extern crate nalgebra;
extern crate nb;
#[macro_use]
extern crate sval;
extern crate sval_json;

pub mod components;
#[macro_use]
pub mod datastructures;
pub mod algorithm;
pub mod config;
pub mod io;
pub mod sync;
#[macro_use]
pub mod sys;
pub mod sysinfo;

#[cfg(test)]
extern crate std;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(test)]
#[macro_use]
extern crate serial_test;
