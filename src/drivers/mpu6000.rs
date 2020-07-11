use core::mem::MaybeUninit;

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use mpu6000::bus::Bus;
use mpu6000::measurement;
use mpu6000::registers::{AccelerometerSensitive, GyroSensitive};
use mpu6000::{self, ClockSource, IntPinConfig, Interrupt, MPU6000};

use crate::alloc;
use crate::datastructures::data_source::overwriting::{OverwritingData, OverwritingDataSource};
use crate::datastructures::data_source::{DataSource, DataWriter};
use crate::datastructures::measurement::{Acceleration, Axes, Gyro, Measurement, Temperature};

pub const ACCELEROMETER_SENSITIVE: AccelerometerSensitive =
    accelerometer_sensitive!(+/-4g, 8192/LSB);
pub const GYRO_SENSITIVE: GyroSensitive = gyro_sensitive!(+/-1000dps, 32.8LSB/dps);

static mut ACCELEROMETER_DATA: MaybeUninit<OverwritingData<Acceleration>> = MaybeUninit::uninit();
static mut GYROSCOPE_DATA: MaybeUninit<OverwritingData<Gyro>> = MaybeUninit::uninit();
static mut TEMPERATURE_DATA: MaybeUninit<OverwritingData<Temperature>> = MaybeUninit::uninit();

impl Into<Measurement> for mpu6000::measurement::Measurement<AccelerometerSensitive> {
    fn into(self) -> Measurement {
        let axis = Axes { x: self.x as i32, y: self.y as i32, z: self.z as i32 };
        let sensitive: f32 = ACCELEROMETER_SENSITIVE.into();
        Measurement { axis, sensitive: sensitive as i32 }
    }
}

impl Into<Measurement> for mpu6000::measurement::Measurement<GyroSensitive> {
    fn into(self) -> Measurement {
        let axis =
            Axes { x: (self.x as i32) << 8, y: (self.y as i32) << 8, z: (self.z as i32) << 8 };
        let sensitive: f32 = GYRO_SENSITIVE.into();
        Measurement { axis, sensitive: (sensitive * 256.0) as i32 }
    }
}

pub fn init_data_source(
) -> (impl DataSource<Acceleration>, impl DataSource<Gyro>, impl DataSource<Temperature>) {
    let buffer = [Acceleration::default(); 40];
    let buffer = alloc::into_static(buffer, false).unwrap();
    unsafe { ACCELEROMETER_DATA = MaybeUninit::new(OverwritingData::new(&mut buffer[..])) };
    let accelerometer = OverwritingDataSource::new(unsafe { &*ACCELEROMETER_DATA.as_ptr() });

    let buffer = [Gyro::default(); 40];
    let buffer = alloc::into_static(buffer, false).unwrap();
    unsafe { GYROSCOPE_DATA = MaybeUninit::new(OverwritingData::new(&mut buffer[..])) };
    let gyroscope = OverwritingDataSource::new(unsafe { &*GYROSCOPE_DATA.as_ptr() });

    let buffer = alloc::into_static([Temperature::default(); 40], false).unwrap();
    unsafe { TEMPERATURE_DATA = MaybeUninit::new(OverwritingData::new(buffer)) };
    let thermometer = OverwritingDataSource::new(unsafe { &*TEMPERATURE_DATA.as_ptr() });

    return (accelerometer, gyroscope, thermometer);
}

pub unsafe fn on_dma_receive(dma_buffer: &[u8; 16]) {
    let buf: &[i16; 8] = core::mem::transmute(dma_buffer);
    let acceleration = measurement::Measurement::from_array(&buf[1..], ACCELEROMETER_SENSITIVE);
    let temperature = measurement::Temperature(i16::from_be(buf[4]));
    let gyro = measurement::Measurement::from_array(&buf[5..], GYRO_SENSITIVE);
    let accelerometer_data = &mut *ACCELEROMETER_DATA.as_mut_ptr();
    accelerometer_data.write(Acceleration(acceleration.into()));
    let gyroscope_data = &mut *GYROSCOPE_DATA.as_mut_ptr();
    gyroscope_data.write(gyro.into());
    let temperature_data = &mut *TEMPERATURE_DATA.as_mut_ptr();
    temperature_data.write(temperature.0);
}

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
