use alloc::boxed::Box;
use core::future::Future;

use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};
use fugit::NanosDurationU64 as Duration;
use hal::dma::{Channel, Peripheral, TransferOption, BD, DMA};
use mpu6000::{
    self,
    bus::RegAccess,
    registers::{AccelerometerSensitive, GyroSensitive, Register},
    ClockSource, IntPinConfig, Interrupt,
};
pub use mpu6000::{bus::SpiBus, MPU6000, SPI_MODE};
use pro_flight::{
    config,
    sys::time::TickTimer,
    types::sensor::{Axes, Readout, Rotation},
};

pub const GYRO_SENSITIVE: GyroSensitive = gyro_sensitive!(+/-1000dps, 32.8LSB/dps);
pub const NUM_MEASUREMENT_REGS: usize = 14;

fn accelerometer_sensitive(sensitive: config::imu::Sensitive) -> AccelerometerSensitive {
    match sensitive.integer() {
        0..=2 => accelerometer_sensitive!(+/-2g, 16384LSB/g),
        3..=4 => accelerometer_sensitive!(+/-4g, 8192LSB/g),
        5..=8 => accelerometer_sensitive!(+/-8g, 4096LSB/g),
        _ => accelerometer_sensitive!(+/-16g, 2048LSB/g),
    }
}

pub struct Converter {
    accelerometer: AccelerometerSensitive,
    gyroscope: GyroSensitive,
}

impl From<config::imu::IMU> for Converter {
    fn from(config: config::imu::IMU) -> Self {
        Self {
            accelerometer: accelerometer_sensitive(config.accelerometer.sensitive),
            gyroscope: GYRO_SENSITIVE,
        }
    }
}

impl Converter {
    fn convert_acceleration(&self, accel: &mpu6000::Acceleration) -> Readout {
        let axes = Axes { x: -accel.0 as i32, y: -accel.1 as i32, z: -accel.2 as i32 };
        let sensitive: f32 = self.accelerometer.into();
        Readout { axes, sensitive: sensitive as u16 }
    }

    fn convert_gyro(&self, gyro: &mpu6000::Gyro) -> Readout {
        let axes =
            Axes { x: (gyro.0 as i32) * 10, y: (gyro.1 as i32) * 10, z: (gyro.2 as i32) * 10 };
        let sensitive: f32 = self.gyroscope.into();
        Readout { axes, sensitive: (sensitive * 10.0) as u16 }
    }

    pub fn convert(&self, bytes: &[u8], rotation: Rotation) -> (Readout, Readout) {
        let acceleration: mpu6000::Acceleration = bytes[..6].into();
        let gyro: mpu6000::Gyro = bytes[8..].into();
        let acceleration = self.convert_acceleration(&acceleration).rotate(rotation);
        let gyro = self.convert_gyro(&gyro).rotate(rotation);
        (acceleration, gyro)
    }
}

pub trait MPU6000Init<E> {
    fn init(&mut self, sample_rate: u16) -> Result<(), E>;
}

impl<E, BUS: RegAccess<Error = E>> MPU6000Init<E> for MPU6000<BUS> {
    fn init(&mut self, sample_rate: u16) -> Result<(), E> {
        let mut delay = TickTimer::default();
        self.reset(&mut delay)?;
        self.set_sleep(false)?;
        delay.delay_us(15u8);
        self.set_i2c_disable(true)?;
        delay.delay_us(15u8);
        self.set_clock_source(ClockSource::PLLGyroZ)?;
        delay.delay_us(15u8);
        let config = config::get().imu;
        self.set_accelerometer_sensitive(accelerometer_sensitive(config.accelerometer.sensitive))?;
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
    rx_bd: Box<BD<u8, { 1 + NUM_MEASUREMENT_REGS }>>,
    tx_bd: Box<BD<u8, 1>>,
}

pub trait IntoDMA<RX: DMA, TX: DMA, CS> {
    fn into_dma(self, rx: (RX, Channel), tx: (TX, Channel)) -> DmaMPU6000<RX, TX, CS>;
}

impl<E, RXF, TXF, SPI, CS, DELAY, RX: DMA, TX: DMA> IntoDMA<RX, TX, CS>
    for MPU6000<SpiBus<SPI, CS, DELAY>>
where
    SPI: Peripheral,
    RX: DMA<Future = RXF>,
    TX: DMA<Future = TXF>,
    CS: OutputPin<Error = E> + Send + Unpin + 'static,
{
    fn into_dma(self, rx: (RX, Channel), tx: (TX, Channel)) -> DmaMPU6000<RX, TX, CS> {
        let (mut spi, cs, _) = self.free().free();
        let (mut rx, ch) = rx;
        rx.setup_peripheral(ch, &mut spi);
        let (mut tx, ch) = tx;
        tx.setup_peripheral(ch, &mut spi);
        let mut rx_bd = Box::new(BD::<u8, { 1 + NUM_MEASUREMENT_REGS }>::default());
        let address = rx_bd.try_get_buffer().unwrap().as_ptr();
        trace!("Init MPU6000 DMA address at 0x{:x}", address as usize);
        let byte = Register::AccelerometerXHigh as u8 | 0x80;
        let tx_bd = Box::new(BD::<u8, 1>::new([byte]));
        DmaMPU6000 { rx, tx, cs, rx_bd, tx_bd }
    }
}

impl<E, O, RXF, TXF, RX, TX, CS> DmaMPU6000<RX, TX, CS>
where
    RXF: Future<Output = O>,
    TXF: Future<Output = O>,
    RX: DMA<Future = RXF>,
    TX: DMA<Future = TXF>,
    CS: OutputPin<Error = E> + Send + Unpin + 'static,
{
    pub async fn run(mut self, mut handler: impl FnMut(Readout, Readout)) {
        let convertor = Converter::from(config::get().imu);
        let rotation = config::get().imu.rotation;
        loop {
            let future = match self.rx.rx(&mut self.rx_bd, Default::default()) {
                Ok(future) => future,
                Err(_) => {
                    TickTimer::after(Duration::millis(1)).await;
                    continue;
                }
            };
            self.cs.set_low().ok();
            self.tx.tx(&self.tx_bd, TransferOption::repeat().size(1 + NUM_MEASUREMENT_REGS)).ok();
            future.await;
            self.cs.set_high().ok();
            if let Some(buffer) = self.rx_bd.try_get_buffer().ok() {
                let (acceleration, gyro) = convertor.convert(&buffer[1..], rotation);
                handler(acceleration, gyro);
            }
        }
    }
}
