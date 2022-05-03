#![cfg_attr(not(any(test, feature = "std")), no_std)]

extern crate alloc;
#[macro_use]
extern crate log;
extern crate nmea0183_core as nmea0183;
#[macro_use]
extern crate serde;

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
pub mod utils;
