use alloc::boxed::Box;

use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::OutputPin;
use hal::dma::{BufferDescriptor, Channel, Peripheral, TransferOption, DMA};
use mpu6000::registers::{AccelerometerSensitive, GyroSensitive, Register};
use mpu6000::{self, bus::RegAccess, ClockSource, IntPinConfig, Interrupt};
pub use mpu6000::{bus::SpiBus, MPU6000};
use pro_flight::config;
use pro_flight::datastructures::measurement::{Acceleration, Axes, Measurement, Rotation};
use pro_flight::sys::timer::SysTimer;

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

pub struct Converter {
    accelerometer: AccelerometerSensitive,
    gyroscope: GyroSensitive,
}

impl Default for Converter {
    fn default() -> Self {
        Self { accelerometer: accelerometer_sensitive(), gyroscope: GYRO_SENSITIVE }
    }
}

impl Converter {
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

    pub fn convert(&self, bytes: &[u8], rotation: Rotation) -> (Acceleration, Measurement) {
        let acceleration: mpu6000::Acceleration = bytes[..6].into();
        let gyro: mpu6000::Gyro = bytes[8..].into();
        let acceleration = Acceleration(self.convert_acceleration(&acceleration).rotate(rotation));
        let gyro = self.convert_gyro(&gyro).rotate(rotation);
        (acceleration, gyro)
    }
}

pub trait MPU6000Init<E> {
    fn init(&mut self, sample_rate: u16) -> Result<(), E>;
}

impl<E, BUS: RegAccess<Error = E>> MPU6000Init<E> for MPU6000<BUS> {
    fn init(&mut self, sample_rate: u16) -> Result<(), E> {
        let mut delay = SysTimer::new();
        self.reset(&mut delay)?;
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

pub struct DmaMPU6000<RX, TX, CS> {
    rx: RX,
    tx: TX,
    cs: CS,
    rx_bd: Box<BufferDescriptor<u8, { 1 + NUM_MEASUREMENT_REGS }>>,
    tx_bd: Box<BufferDescriptor<u8, 1>>,
}

pub trait IntoDMA<RX: DMA, TX: DMA, CS> {
    fn into_dma(self, rx: (RX, Channel), tx: (TX, Channel)) -> DmaMPU6000<RX, TX, CS>;
}

impl<E, SPI, CS, DELAY, RX: DMA, TX: DMA> IntoDMA<RX, TX, CS> for MPU6000<SpiBus<SPI, CS, DELAY>>
where
    SPI: Peripheral,
    CS: OutputPin<Error = E> + Send + Unpin + 'static,
{
    fn into_dma(self, rx: (RX, Channel), tx: (TX, Channel)) -> DmaMPU6000<RX, TX, CS> {
        let (mut spi, cs, _) = self.free().free();
        let (mut rx, ch) = rx;
        rx.setup_peripheral(ch, &mut spi);
        let (mut tx, ch) = tx;
        tx.setup_peripheral(ch, &mut spi);
        let mut rx_bd = Box::new(BufferDescriptor::<u8, { 1 + NUM_MEASUREMENT_REGS }>::default());
        let mut cs_ = unsafe { core::ptr::read(&cs as *const _ as *const CS) };
        rx_bd.set_transfer_done(move |_bytes| {
            cs_.set_high().ok();
        });
        let address = rx_bd.try_get_buffer().unwrap().as_ptr();
        debug!("Init MPU6000 DMA address at 0x{:x}", address as usize);
        let byte = Register::AccelerometerXHigh as u8 | 0x80;
        let tx_bd = Box::new(BufferDescriptor::<u8, 1>::new([byte]));
        DmaMPU6000 { rx, tx, cs, rx_bd, tx_bd }
    }
}

impl<E, RXF, TXF, RX, TX, CS> DmaMPU6000<RX, TX, CS>
where
    RX: DMA<Future = RXF>,
    TX: DMA<Future = TXF>,
    CS: OutputPin<Error = E> + Send + Unpin + 'static,
{
    pub fn set_handler<F>(&mut self, mut handler: F)
    where
        F: FnMut(Acceleration, Measurement) + Send + 'static,
    {
        let mut cs = unsafe { core::ptr::read(&self.cs as *const _ as *const CS) };
        let convertor = Converter::default();
        let rotation = config::get().board.rotation;
        self.rx_bd.set_transfer_done(move |bytes| {
            cs.set_high().ok();
            let (acceleration, gyro) = convertor.convert(&bytes[1..], rotation);
            handler(acceleration, gyro);
        });
    }

    pub fn trigger(&mut self) {
        self.cs.set_low().ok();
        if self.rx.is_busy() || self.tx.is_busy() {
            return;
        }
        self.rx.rx(&self.rx_bd, Default::default());
        self.tx.tx(&self.tx_bd, TransferOption::repeat(1 + NUM_MEASUREMENT_REGS));
    }
}
