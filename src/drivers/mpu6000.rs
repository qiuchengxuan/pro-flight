use core::mem::MaybeUninit;

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use mpu6000::bus::Bus;
use mpu6000::measurement;
use mpu6000::registers::{AccelerometerSensitive, GyroSensitive};
use mpu6000::{self, ClockSource, IntPinConfig, Interrupt, MPU6000};

use crate::alloc::{self, AllocateType};
use crate::datastructures::ring_buffer::{RingBuffer, RingBufferReader};
use crate::hal::sensors::{Acceleration, Axis, Gyro, Measurement, Temperature};

pub const ACCELEROMETER_SENSITIVE: AccelerometerSensitive =
    accelerometer_sensitive!(+/-4g, 8192/LSB);
pub const GYRO_SENSITIVE: GyroSensitive = gyro_sensitive!(+/-1000dps, 32.8LSB/dps);

type AccelGyro = (Acceleration, Gyro);

static mut ACCEL_GYRO_RING: MaybeUninit<RingBuffer<AccelGyro>> = MaybeUninit::uninit();
static mut TEMPERATURE_RING: MaybeUninit<RingBuffer<Temperature>> = MaybeUninit::uninit();

impl Into<Measurement> for mpu6000::measurement::Measurement<AccelerometerSensitive> {
    fn into(self) -> Measurement {
        let axes = Axis { x: self.x as i32, y: self.y as i32, z: self.z as i32 };
        let sensitive: f32 = ACCELEROMETER_SENSITIVE.into();
        Measurement { axes, sensitive: sensitive as i32 }
    }
}

impl Into<Measurement> for mpu6000::measurement::Measurement<GyroSensitive> {
    fn into(self) -> Measurement {
        let axes =
            Axis { x: (self.x as i32) << 8, y: (self.y as i32) << 8, z: (self.z as i32) << 8 };
        let sensitive: f32 = GYRO_SENSITIVE.into();
        Measurement { axes, sensitive: (sensitive * 256.0) as i32 }
    }
}

pub fn init_accel_gyro_ring() -> RingBufferReader<'static, AccelGyro> {
    let buffer = [(Acceleration::default(), Gyro::default()); 40];
    let buffer = alloc::into_static(buffer, alloc::AllocateType::Generic).unwrap();
    unsafe { ACCEL_GYRO_RING = MaybeUninit::new(RingBuffer::new(buffer)) };
    RingBufferReader::new(unsafe { &*ACCEL_GYRO_RING.as_ptr() })
}

pub fn init_temperature_ring() -> RingBufferReader<'static, Temperature> {
    let buffer = alloc::into_static([Temperature::default(); 40], AllocateType::Generic).unwrap();
    unsafe { TEMPERATURE_RING = MaybeUninit::new(RingBuffer::new(buffer)) };
    RingBufferReader::new(unsafe { &*TEMPERATURE_RING.as_ptr() })
}

pub unsafe fn on_dma_receive(dma_buffer: &[u8; 16]) {
    let buf: &[i16; 8] = core::mem::transmute(dma_buffer);
    let acceleration = measurement::Measurement::from_array(&buf[1..], ACCELEROMETER_SENSITIVE);
    let temperature = measurement::Temperature(i16::from_be(buf[4]));
    let gyro = measurement::Measurement::from_array(&buf[5..], GYRO_SENSITIVE);
    let ring_buffer = &mut *ACCEL_GYRO_RING.as_mut_ptr();
    ring_buffer.write((acceleration.into(), gyro.into()));
    let ring_buffer = &mut *TEMPERATURE_RING.as_mut_ptr();
    ring_buffer.write(temperature.0);
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
