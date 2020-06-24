#![no_std]
#![feature(maybe_uninit_uninit_array)]
#![feature(trait_alias)]

extern crate ahrs;
extern crate arrayvec;
extern crate ascii;
extern crate ascii_osd_hud;
extern crate bmp280;
extern crate btoi;
#[macro_use]
extern crate enum_map;
extern crate embedded_sdmmc;
extern crate integer_sqrt;
#[macro_use]
extern crate log;
extern crate micromath;
#[macro_use]
extern crate mpu6000;
extern crate nalgebra;
extern crate nb;
extern crate ryu;
extern crate sbus_parser;
#[macro_use]
extern crate sval;
extern crate sval_json;

#[macro_use]
pub mod components;
pub mod config;
pub mod datastructures;
pub mod drivers;
pub mod hal;
pub mod sys;

#[cfg(test)]
extern crate std;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
