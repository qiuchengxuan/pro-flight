#![cfg_attr(not(test), no_std)]
#![feature(trait_alias)]

extern crate nb;

#[macro_use]
pub mod sys;
pub mod drivers;
