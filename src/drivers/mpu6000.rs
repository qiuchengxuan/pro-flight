use embedded_hal::blocking::delay::DelayUs;
use mpu6000::bus::Bus;
use mpu6000::registers::{AccelerometerSensitive, GyroSensitive};
use mpu6000::{self, ClockSource, IntPinConfig, Interrupt, MPU6000};

use crate::config;
use crate::datastructures::measurement::{Acceleration, Axes, Measurement, Rotation};
use crate::sys::timer::SysTimer;

pub const GYRO_SENSITIVE: GyroSensitive = gyro_sensitive!(+/-1000dps, 32.8LSB/dps);
pub const NUM_MEASUREMENT_REGS: usize = 14;

fn accelerometer_sensitive() -> AccelerometerSensitive {
    let imu = config::get().imu;
    match imu.accelerometer.sensitive.integer() {
        0..=2 => accelerometer_sensitive!(+/-2g, 16384/LSB),
        3..=4 => accelerometer_sensitive!(+/-4g, 8192/LSB),
        5..=8 => accelerometer_sensitive!(+/-8g, 4096/LSB),
        _ => accelerometer_sensitive!(+/-16g, 2048/LSB),
    }
}

pub struct Convertor {
    accelerometer: AccelerometerSensitive,
    gyroscope: GyroSensitive,
}

impl Default for Convertor {
    fn default() -> Self {
        Self { accelerometer: accelerometer_sensitive(), gyroscope: GYRO_SENSITIVE }
    }
}

impl Convertor {
    fn convert_acceleration(&self, accel: &mpu6000::Acceleration) -> Measurement {
        let axes = Axes { x: -accel.0 as i32, y: -accel.1 as i32, z: -accel.2 as i32 };
        let sensitive: f32 = self.accelerometer.into();
        Measurement { axes, sensitive: sensitive as i32 }
    }

    fn convert_gyro(&self, gyro: &mpu6000::Gyro) -> Measurement {
        let axes =
            Axes { x: (gyro.0 as i32) << 8, y: (gyro.1 as i32) << 8, z: (gyro.2 as i32) << 8 };
        let sensitive: f32 = self.gyroscope.into();
        Measurement { axes, sensitive: (sensitive * 256.0) as i32 }
    }

    pub fn convert(&self, data: &[i16; 7], rotation: Rotation) -> (Acceleration, Measurement) {
        let acceleration: mpu6000::Acceleration = data[..3].into();
        let gyro: mpu6000::Gyro = data[4..].into();
        let acceleration = Acceleration(self.convert_acceleration(&acceleration).rotate(rotation));
        let gyro = self.convert_gyro(&gyro).rotate(rotation);
        (acceleration, gyro)
    }
}

pub trait MPU6000Init<E> {
    fn init(&mut self, sample_rate: u16) -> Result<(), E>;
}

impl<E, BUS: Bus<Error = E>> MPU6000Init<E> for MPU6000<BUS> {
    fn init(&mut self, sample_rate: u16) -> Result<(), E> {
        let mut delay = SysTimer::new();
        self.set_sleep(false)?;
        delay.delay_us(15u8);
        self.set_i2c_disable(true)?;
        delay.delay_us(15u8);
        self.set_clock_source(ClockSource::PLLGyroZ)?;
        delay.delay_us(15u8);
        self.set_accelerometer_sensitive(accelerometer_sensitive())?;
        delay.delay_us(15u8);
        self.set_gyro_sensitive(GYRO_SENSITIVE)?;
        delay.delay_us(15u8);
        self.set_dlpf(2)?;
        delay.delay_us(15u8);
        self.set_sample_rate(sample_rate)?;
        delay.delay_us(15u8);
        self.set_int_pin_config(IntPinConfig::IntReadClear, true)?;
        delay.delay_us(15u8);
        self.set_interrupt_enable(Interrupt::DataReady, true)?;
        Ok(())
    }
}
