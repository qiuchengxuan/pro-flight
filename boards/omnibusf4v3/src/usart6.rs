use core::mem::MaybeUninit;

use rs_flight::config::SerialConfig;
use rs_flight::drivers::sbus::SbusReceiver;
use rs_flight::drivers::uart::Device;
use stm32f4xx_hal::gpio::gpioc;
use stm32f4xx_hal::gpio::{Alternate, Floating, Input, AF8};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::serial::config::{Config, StopBits};
use stm32f4xx_hal::serial::Serial;
use stm32f4xx_hal::{prelude::*, stm32};

type PC6 = gpioc::PC6<Input<Floating>>;
type PC7 = gpioc::PC7<Input<Floating>>;
type PINS = (gpioc::PC6<Alternate<AF8>>, gpioc::PC7<Alternate<AF8>>);

const HTIF_OFFSET: usize = 4;
const STREAM1_OFFSET: usize = 6;

#[export_name = "USART6_DMA_BUFFER"]
static mut DMA_BUFFER: [u8; 64] = [0u8; 64];
static mut DEVICE: Device = Device::None;
static mut USART6: MaybeUninit<Serial<stm32::USART6, PINS>> = MaybeUninit::uninit();

#[interrupt]
unsafe fn DMA2_STREAM1() {
    let mut half = false;
    cortex_m::interrupt::free(|_| {
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM1);
        let dma2 = &*stm32::DMA2::ptr();
        half = (dma2.lisr.read().bits() & (1 << HTIF_OFFSET) << STREAM1_OFFSET) > 0;
        dma2.lifcr.write(|w| w.bits(0x3D << STREAM1_OFFSET));
    });
    let offset = if half { 0 } else { DMA_BUFFER.len() / 2 };
    DEVICE.handle(&DMA_BUFFER, offset, DMA_BUFFER.len() / 2);
}

pub fn init(
    usart6: stm32::USART6,
    pins: (PC6, PC7),
    config: &SerialConfig,
    clocks: Clocks,
) -> &'static mut Device {
    unsafe { DEVICE = Device::None };

    let mut cfg = Config::default();
    match config {
        SerialConfig::SBUS(sbus) => {
            debug!("Config USART6 as SBUS receiver");
            cfg = cfg.baudrate(sbus.baudrate().bps());
            // word-length-9 must be selected when parity is even
            cfg = cfg.stopbits(StopBits::STOP2).parity_even().wordlength_9();
        }
        _ => return unsafe { &mut DEVICE },
    };

    let (pc6, pc7) = pins;
    let pins = (pc6.into_alternate_af8(), pc7.into_alternate_af8());
    let usart = Serial::usart6(usart6, pins, cfg, clocks).unwrap();

    // dma2 stream1 channel 5 rx
    unsafe {
        USART6 = MaybeUninit::new(usart);
        (&*stm32::USART6::ptr()).cr3.modify(|_, w| w.dmar().enabled());

        let dma2 = &*(stm32::DMA2::ptr());
        let stream = &dma2.st[1];
        stream.ndtr.write(|w| w.ndt().bits(DMA_BUFFER.len() as u16));
        stream.par.write(|w| w.pa().bits(&(&*(stm32::USART6::ptr())).dr as *const _ as u32));
        let m0ar = &stream.m0ar;
        m0ar.write(|w| w.m0a().bits(DMA_BUFFER.as_ptr() as u32));
        #[rustfmt::skip]
        stream.cr.write(|w| {
            w.chsel().bits(5).minc().incremented().dir().peripheral_to_memory().circ().enabled()
                .pl().very_high().htie().enabled().tcie().enabled().en().enabled()
        });
    }

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM1);
    unsafe {
        cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA2_STREAM1);
    }
    // FIXME: use a timer to ensure remaining data be received

    let device = match config {
        SerialConfig::SBUS(_) => Device::SBUS(SbusReceiver::new()),
        _ => Device::None,
    };
    unsafe {
        DEVICE = device;
        &mut DEVICE
    }
}
