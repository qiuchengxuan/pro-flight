use core::fmt::Write;

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use mpu6000::bus::Bus;
use mpu6000::registers::{AccelerometerSensitive, GyroSensitive};
use mpu6000::{self, ClockSource, IntPinConfig, Interrupt, MPU6000};

use crate::components::logger::Logger;
use crate::hal::sensors::Measurement;

impl Into<Measurement> for mpu6000::measurement::Measurement<AccelerometerSensitive> {
    fn into(self) -> Measurement {
        Measurement { x: self.x, y: self.y, z: self.z, sensitive: self.sensitive.into() }
    }
}

impl Into<Measurement> for mpu6000::measurement::Measurement<GyroSensitive> {
    fn into(self) -> Measurement {
        Measurement { x: self.x, y: self.y, z: self.z, sensitive: self.sensitive.into() }
    }
}

pub const ACCELEROMETER_SENSITIVE: AccelerometerSensitive =
    accelerometer_sensitive!(+/-4g, 8192/LSB);
pub const GYRO_SENSITIVE: GyroSensitive = gyro_sensitive!(+/-1000dps, 32.8LSB/dps);

pub fn init<E, BUS, D>(bus: BUS, sample_rate: u16, delay: &mut D) -> Result<bool, E>
where
    BUS: Bus<Error = E>,
    D: DelayUs<u8> + DelayMs<u8>,
{
    let mut mpu6000 = MPU6000::new(bus);
    mpu6000.reset(delay)?;
    if !mpu6000.verify()? {
        return Ok(false);
    }
    log!("MPU6000 detected");
    mpu6000.set_sleep(false)?;
    delay.delay_us(15u8);
    mpu6000.set_i2c_disable(true)?;
    delay.delay_us(15u8);
    mpu6000.set_clock_source(ClockSource::PLLGyroZ)?;
    delay.delay_us(15u8);
    mpu6000.set_accelerometer_sensitive(ACCELEROMETER_SENSITIVE)?;
    delay.delay_us(15u8);
    mpu6000.set_gyro_sensitive(GYRO_SENSITIVE)?;
    delay.delay_us(15u8);
    mpu6000.set_dlpf(1)?;
    delay.delay_us(15u8);
    mpu6000.set_sample_rate(sample_rate)?;
    delay.delay_us(15u8);
    mpu6000.set_int_pin_config(IntPinConfig::IntReadClear, true)?;
    delay.delay_us(15u8);
    mpu6000.set_interrupt_enable(Interrupt::DataReady, true)?;
    Ok(true)
}
