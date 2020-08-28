use alloc::rc::Rc;
use core::mem::MaybeUninit;

use embedded_hal::blocking::delay::DelayUs;
use mpu6000::bus::Bus;
use mpu6000::measurement;
use mpu6000::registers::{AccelerometerSensitive, GyroSensitive};
use mpu6000::{self, ClockSource, IntPinConfig, Interrupt, MPU6000};

use crate::alloc;
use crate::datastructures::data_source::overwriting::{OverwritingData, OverwritingDataSource};
use crate::datastructures::data_source::DataWriter;
use crate::datastructures::measurement::{Acceleration, Axes, Gyro, Measurement, Temperature};
use crate::sys::timer::SysTimer;

pub const ACCELEROMETER_SENSITIVE: AccelerometerSensitive =
    accelerometer_sensitive!(+/-4g, 8192/LSB);
pub const GYRO_SENSITIVE: GyroSensitive = gyro_sensitive!(+/-1000dps, 32.8LSB/dps);

pub struct MPU6000Data {
    accelerometer: Rc<OverwritingData<Acceleration>>,
    gyroscope: Rc<OverwritingData<Gyro>>,
    thermometer: Rc<OverwritingData<Temperature>>,
}

static mut MPU6000_DATA: MaybeUninit<MPU6000Data> = MaybeUninit::uninit();

impl Into<Measurement> for mpu6000::measurement::Measurement<AccelerometerSensitive> {
    fn into(self) -> Measurement {
        let axes = Axes { x: -self.x as i32, y: -self.y as i32, z: -self.z as i32 };
        let sensitive: f32 = ACCELEROMETER_SENSITIVE.into();
        Measurement { axes, sensitive: sensitive as i32 }
    }
}

impl Into<Measurement> for mpu6000::measurement::Measurement<GyroSensitive> {
    fn into(self) -> Measurement {
        let axes =
            Axes { x: (self.x as i32) << 8, y: (self.y as i32) << 8, z: (self.z as i32) << 8 };
        let sensitive: f32 = GYRO_SENSITIVE.into();
        Measurement { axes, sensitive: (sensitive * 256.0) as i32 }
    }
}

pub fn init_data_source() -> (
    OverwritingDataSource<Acceleration>,
    OverwritingDataSource<Gyro>,
    OverwritingDataSource<Temperature>,
) {
    let mpu6000_data = MPU6000Data {
        accelerometer: Rc::new(OverwritingData::sized(40)),
        gyroscope: Rc::new(OverwritingData::sized(40)),
        thermometer: Rc::new(OverwritingData::sized(40)),
    };
    unsafe { MPU6000_DATA = MaybeUninit::new(mpu6000_data) };

    let mpu6000_data = unsafe { &*MPU6000_DATA.as_ptr() };
    let accelerometer = OverwritingDataSource::new(&mpu6000_data.accelerometer);
    let gyroscope = OverwritingDataSource::new(&mpu6000_data.gyroscope);
    let thermometer = OverwritingDataSource::new(&mpu6000_data.thermometer);
    return (accelerometer, gyroscope, thermometer);
}

pub unsafe fn on_dma_receive(dma_buffer: &[u8; 16]) {
    let buf: &[i16; 8] = core::mem::transmute(dma_buffer);
    let acceleration = measurement::Measurement::from_array(&buf[1..], ACCELEROMETER_SENSITIVE);
    let temperature = measurement::Temperature(i16::from_be(buf[4]));
    let gyro = measurement::Measurement::from_array(&buf[5..], GYRO_SENSITIVE);
    let mpu6000_data = &mut *MPU6000_DATA.as_mut_ptr();
    mpu6000_data.accelerometer.write(Acceleration(acceleration.into()));
    mpu6000_data.gyroscope.write(gyro.into());
    mpu6000_data.thermometer.write(temperature.0);
}

pub fn init<E, BUS: Bus<Error = E>>(bus: BUS, sample_rate: u16) -> Result<bool, E> {
    let mut mpu6000 = MPU6000::new(bus);
    let mut delay = SysTimer::new();
    mpu6000.reset(&mut delay)?;
    if !mpu6000.verify()? {
        return Ok(false);
    }
    info!("MPU6000 detected");
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
