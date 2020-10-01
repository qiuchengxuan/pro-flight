use alloc::boxed::Box;

use stm32f4xx_hal::gpio::gpiob;
use stm32f4xx_hal::gpio::{Floating, Input};
use stm32f4xx_hal::i2c::{Error, I2c};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::{prelude::*, stm32};

use rs_flight::components::schedule::{Rate, Schedulable};
use rs_flight::drivers::magnetometer::qmc5883l;

const STREAM7_OFFSET: usize = 22;
const STREAM3_OFFSET: usize = 22;

type PB10 = gpiob::PB10<Input<Floating>>;
type PB11 = gpiob::PB11<Input<Floating>>;

#[interrupt]
unsafe fn DMA1_STREAM3() {
    let mut buffer: &[u8] = &[0u8; 0];
    cortex_m::interrupt::free(|_| {
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA1_STREAM3);
        let dma1 = &*stm32::DMA1::ptr();
        dma1.hifcr.write(|w| w.bits(0x3D << STREAM7_OFFSET));
        dma1.lifcr.write(|w| w.bits(0x3D << STREAM3_OFFSET));
        let address = dma1.st[3].m0ar.read().bits();
        let size = *((address - 2) as *const u16) as usize;
        buffer = core::slice::from_raw_parts(address as *const _, size);
    });

    qmc5883l::on_dma_receive(buffer);
}

fn i2c_start_tx(bytes: &[u8]) {
    let dma1 = unsafe { &*(stm32::DMA1::ptr()) };
    unsafe { dma1.hifcr.write(|w| w.bits(0x3D << STREAM7_OFFSET)) };
    let stream = &dma1.st[7]; // dma1 channel 7 stream 7 tx
    stream.m0ar.write(|w| w.m0a().bits(bytes.as_ptr() as u32));
    stream.ndtr.write(|w| w.ndt().bits(bytes.len() as u16));
    stream.cr.modify(|_, w| w.minc().incremented().en().enabled());
}

pub struct MagnetometerScheduler;

impl Schedulable for MagnetometerScheduler {
    fn rate(&self) -> Rate {
        200
    }

    fn schedule(&mut self) -> bool {
        i2c_start_tx(qmc5883l::dma_read_bytes());
        true
    }
}

fn init_dma() {
    let i2c2 = unsafe { &*stm32::I2C2::ptr() };
    i2c2.cr2.modify(|_, w| w.dmaen().enabled());
    let data_register = &i2c2.dr as *const _ as u32;

    let dma_buffer = Box::leak(Box::new([0i16; 4]));
    let address = dma_buffer.as_ptr() as usize + 2;
    let size = dma_buffer.len() - 2;
    unsafe { *(dma_buffer as *mut _ as *mut u16) = size as u16 };
    debug!("Alloc DMA buffer at {:#X} size {} on I2C2", address, size);

    let dma1 = unsafe { &*(stm32::DMA1::ptr()) };
    // dma1 stream 3 rx
    let stream = &dma1.st[3];
    stream.par.write(|w| w.pa().bits(data_register));
    stream.m0ar.write(|w| w.m0a().bits(address as u32 + 1));
    #[rustfmt::skip]
    stream.cr.write(|w| {
        w.chsel().bits(7).minc().incremented().dir().peripheral_to_memory()
            .pburst().incr16().tcie().enabled()
    });

    // dma1 stream 7 tx
    let stream = &dma1.st[7];
    stream.par.write(|w| w.pa().bits(data_register));
    stream.m0ar.write(|w| w.m0a().bits(&qmc5883l::dma_read_bytes() as *const _ as u32));
    stream.cr.write(|w| w.chsel().bits(7).dir().memory_to_peripheral().pburst().incr16());
}

pub fn init(
    i2c2: stm32::I2C2,
    i2c2_pins: (PB10, PB11),
    clocks: Clocks,
) -> Result<impl Schedulable, Error> {
    let (pb10, pb11) = i2c2_pins;
    let scl = pb10.into_alternate_af4().set_open_drain();
    let sda = pb11.into_alternate_af4().set_open_drain();
    let i2c2 = I2c::i2c2(i2c2, (scl, sda), 400.khz(), clocks);
    // TODO: scan i2c bus
    qmc5883l::init(i2c2)?;
    init_dma();

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA1_STREAM3);
    unsafe {
        cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA1_STREAM3);
    }
    Ok(MagnetometerScheduler)
}
