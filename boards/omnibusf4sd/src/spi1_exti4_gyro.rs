use core::convert::Infallible;
use core::mem::MaybeUninit;

use mpu6000::bus::{self, DelayNs, SpiBus};
use mpu6000::measurement;
use mpu6000::registers::Register;
use mpu6000::SPI_MODE;

use rs_flight::datastructures::ring_buffer::{RingBuffer, RingBufferReader};
use rs_flight::drivers::mpu6000::{init as mpu6000_init, ACCELEROMETER_SENSITIVE, GYRO_SENSITIVE};
use rs_flight::hal::sensors::{Acceleration, Gyro, Temperature};
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::gpioa::{PA4, PA5, PA6, PA7};
use stm32f4xx_hal::gpio::gpioc::PC4;
use stm32f4xx_hal::gpio::ExtiPin;
use stm32f4xx_hal::gpio::{Floating, Input, Output, PullUp, PushPull};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Spi};
use stm32f4xx_hal::{prelude::*, stm32};

pub struct TickDelay(u32);

impl DelayNs<u8> for TickDelay {
    fn delay_ns(&mut self, ns: u8) {
        cortex_m::asm::delay(ns as u32 * (self.0 / 1000_000) / 1000 + 1)
    }
}

impl DelayNs<u16> for TickDelay {
    fn delay_ns(&mut self, ns: u16) {
        cortex_m::asm::delay(ns as u32 * (self.0 / 1000_000) / 1000 + 1)
    }
}

type SpiError = bus::SpiError<Error, Error, Infallible>;
type AccelGyro = (Acceleration, Gyro);

static mut CS: MaybeUninit<PA4<Output<PushPull>>> = MaybeUninit::uninit();
static mut INT: MaybeUninit<PC4<Input<PullUp>>> = MaybeUninit::uninit();
#[export_name = "GYRO_DMA_BUFFER"]
static mut DMA_BUFFER: [u8; 16] = [0u8; 16];
static mut ACCEL_GYRO_RING: MaybeUninit<RingBuffer<AccelGyro>> = MaybeUninit::uninit();
static mut TEMPERATURE_RING: MaybeUninit<RingBuffer<Temperature>> = MaybeUninit::uninit();

#[interrupt]
unsafe fn EXTI4() {
    cortex_m::interrupt::free(|_| {
        { &mut *INT.as_mut_ptr() }.clear_interrupt_pending_bit();
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI4);
    });

    { &mut *CS.as_mut_ptr() }.set_low().ok();

    let spi1 = &(*stm32::SPI1::ptr());
    let data_register = &spi1.dr as *const _ as u32;
    let dma2 = &*(stm32::DMA2::ptr());

    // dma2 channel 3 stream 0 rx
    let stream = &dma2.st[0];
    stream.ndtr.write(|w| w.ndt().bits(DMA_BUFFER.len() as u16));
    stream.par.write(|w| w.pa().bits(data_register));
    let m0ar = &stream.m0ar;
    m0ar.write(|w| w.m0a().bits(DMA_BUFFER.as_ptr() as u32));
    #[rustfmt::skip]
    stream.cr.write(|w| {
        w.chsel().bits(3).minc().incremented().dir().peripheral_to_memory()
            .tcie().enabled().en().enabled()
    });

    static READ_REG: [u8; 1] = [Register::AccelerometerXHigh as u8 | 0x80];

    // dma2 channel 3 stream 3 tx
    let stream = &dma2.st[3];
    stream.ndtr.write(|w| w.ndt().bits(DMA_BUFFER.len() as u16));
    stream.par.write(|w| w.pa().bits(data_register));
    stream.m0ar.write(|w| w.m0a().bits(READ_REG.as_ptr() as u32));
    let cr = &stream.cr;
    cr.write(|w| w.chsel().bits(3).dir().memory_to_peripheral().en().enabled());
}

#[interrupt]
unsafe fn DMA2_STREAM0() {
    cortex_m::interrupt::free(|_| {
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM0);
        let dma2 = &*stm32::DMA2::ptr();
        dma2.lifcr.write(|w| w.bits(0x3D << 22 | 0x3D));
    });

    let buf = &DMA_BUFFER;
    let acceleration = measurement::Measurement::from_bytes(&buf[1..], ACCELEROMETER_SENSITIVE);
    let temperature = measurement::Temperature(i16::from_be_bytes([buf[7], buf[8]]));
    let gyro = measurement::Measurement::from_bytes(&buf[9..], GYRO_SENSITIVE);
    let ring_buffer = &mut *ACCEL_GYRO_RING.as_mut_ptr();
    ring_buffer.write((acceleration.into(), gyro.into()));
    let ring_buffer = &mut *TEMPERATURE_RING.as_mut_ptr();
    ring_buffer.write(temperature.0);
    { &mut *CS.as_mut_ptr() }.set_high().ok();
}

pub fn init_accel_gyro_ring() -> RingBufferReader<'static, AccelGyro> {
    #[link_section = ".ccmram"]
    static mut ACCEL_GYRO_BUFFER: MaybeUninit<[AccelGyro; 40]> = MaybeUninit::uninit();
    unsafe {
        ACCEL_GYRO_RING = MaybeUninit::new(RingBuffer::new(&mut *ACCEL_GYRO_BUFFER.as_mut_ptr()))
    };
    RingBufferReader::new(unsafe { &*ACCEL_GYRO_RING.as_ptr() })
}

pub fn init_temperature_ring() -> RingBufferReader<'static, Temperature> {
    #[link_section = ".ccmram"]
    static mut TEMPERATURE_BUFFER: MaybeUninit<[Temperature; 40]> = MaybeUninit::uninit();
    unsafe {
        TEMPERATURE_RING = MaybeUninit::new(RingBuffer::new(&mut *TEMPERATURE_BUFFER.as_mut_ptr()));
    }
    RingBufferReader::new(unsafe { &*TEMPERATURE_RING.as_ptr() })
}

pub fn init(
    spi1: stm32::SPI1,
    spi1_pins: (PA5<Input<Floating>>, PA6<Input<Floating>>, PA7<Input<Floating>>),
    pa4: PA4<Input<Floating>>,
    int: PC4<Input<PullUp>>,
    clocks: Clocks,
    delay: &mut Delay,
    sample_rate: u16,
) -> Result<bool, SpiError> {
    let mut cs = pa4.into_push_pull_output();
    let (pa5, pa6, pa7) = spi1_pins;
    let sclk = pa5.into_alternate_af5();
    let miso = pa6.into_alternate_af5();
    let mosi = pa7.into_alternate_af5();

    let freq: stm32f4xx_hal::time::Hertz = 1.mhz().into();
    let spi1 = Spi::spi1(spi1, (sclk, miso, mosi), SPI_MODE, freq, clocks);
    let bus = SpiBus::new(spi1, &mut cs, TickDelay(clocks.sysclk().0));
    if !mpu6000_init(bus, sample_rate, delay)? {
        return Ok(false);
    }

    let freq: stm32f4xx_hal::time::Hertz = 20.mhz().into();
    let br = match clocks.pclk2().0 / freq.0 {
        0 => unreachable!(),
        1..=2 => 0b000,
        3..=5 => 0b001,
        6..=11 => 0b010,
        12..=23 => 0b011,
        24..=47 => 0b100,
        48..=95 => 0b101,
        96..=191 => 0b110,
        _ => 0b011,
    };
    let spi1 = unsafe { &(*stm32::SPI1::ptr()) };
    spi1.cr1.modify(|_, w| w.br().bits(br));
    spi1.cr2.modify(|_, w| w.txdmaen().enabled().rxdmaen().enabled());

    unsafe {
        CS = MaybeUninit::new(cs);
        INT = MaybeUninit::new(int);
    }

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM0);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA2_STREAM0) }
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI4);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::EXTI4) }
    Ok(true)
}
