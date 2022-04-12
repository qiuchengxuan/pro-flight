#![cfg_attr(not(any(test, feature = "std")), no_std)]

extern crate alloc;
extern crate ascii;
extern crate ascii_osd_hud;
extern crate crc16;
extern crate derive_more;
extern crate heapless;
extern crate integer_sqrt;
extern crate libm;
#[macro_use]
extern crate log;
extern crate micromath;
extern crate nalgebra;
extern crate nb;
extern crate nmea0183_core as nmea0183;
extern crate sbus_parser;
#[macro_use]
extern crate serde;
extern crate spin;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
#[cfg(test)]
#[macro_use]
extern crate serial_test;

#[macro_use]
pub mod io;
#[macro_use]
pub mod types;
#[macro_use]
pub mod sys;

pub mod algorithm;
pub mod cli;
pub mod collection;
pub mod config;
pub mod datastore;
pub mod fcs;
pub mod imu;
pub mod ins;
pub mod logger;
pub mod osd;
pub mod protocol;
pub mod servo;
pub mod sync;
pub mod sysinfo;
pub mod task;
