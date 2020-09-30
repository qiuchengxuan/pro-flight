use alloc::vec::Vec;

use rs_flight::config::SerialConfig;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::serial::config::{Config, StopBits};

pub fn to_serial_config(config: &SerialConfig) -> Config {
    match config {
        SerialConfig::GNSS(gnss) => Config::default().baudrate(gnss.baudrate.bps()),
        SerialConfig::SBUS(sbus) => Config::default()
            .baudrate(sbus.baudrate().bps())
            .stopbits(StopBits::STOP2)
            .parity_even()
            .wordlength_9(),
        _ => Config::default(),
    }
}

pub fn alloc_by_config(config: &SerialConfig) -> Vec<u8> {
    match config {
        SerialConfig::GNSS(_) => vec![0u8; 128],
        SerialConfig::SBUS(_) => vec![0u8; 64],
        _ => vec![0u8; 64],
    }
}
