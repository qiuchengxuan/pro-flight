use core::mem::MaybeUninit;

use rs_flight::config::SerialConfig;
use rs_flight::drivers::gnss::GNSS;
use rs_flight::drivers::uart::Device;
use stm32f4xx_hal::gpio::gpioa;
use stm32f4xx_hal::gpio::{Alternate, Floating, Input, AF7};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::serial::config::Config;
use stm32f4xx_hal::serial::Serial;
use stm32f4xx_hal::{prelude::*, stm32};

type PA9 = gpioa::PA9<Input<Floating>>;
type PA10 = gpioa::PA10<Input<Floating>>;
type PINS = (gpioa::PA9<Alternate<AF7>>, gpioa::PA10<Alternate<AF7>>);

const HTIF_OFFSET: usize = 4;
const STREAM5_OFFSET: usize = 6;

static mut DMA_BUFFER: [u8; 64] = [0u8; 64];
static mut DEVICE: Device = Device::None;
static mut USART1: MaybeUninit<Serial<stm32::USART1, PINS>> = MaybeUninit::uninit();

#[interrupt]
unsafe fn DMA2_STREAM5() {
    let mut half = false;
    cortex_m::interrupt::free(|_| {
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM5);
        let dma2 = &*stm32::DMA2::ptr();
        half = (dma2.hisr.read().bits() & (1 << HTIF_OFFSET) << STREAM5_OFFSET) > 0;
        dma2.hifcr.write(|w| w.bits(0x3D << STREAM5_OFFSET));
    });
    DEVICE.handle(&DMA_BUFFER, half, DMA_BUFFER.len() / 2);
}

pub fn init(
    usart1: stm32::USART1,
    pins: (PA9, PA10),
    config: &SerialConfig,
    clocks: Clocks,
) -> &'static mut Device {
    unsafe { DEVICE = Device::None };

    let mut cfg = Config::default();
    match config {
        SerialConfig::GNSS(gnss) => {
            debug!("Config USART6 as GNSS receiver");
            cfg = cfg.baudrate(gnss.baudrate.bps());
        }
        _ => return unsafe { &mut DEVICE },
    };

    let (pa9, pa10) = pins;
    let pins = (pa9.into_alternate_af7(), pa10.into_alternate_af7());
    let usart = Serial::usart1(usart1, pins, cfg, clocks).unwrap();

    unsafe {
        USART1 = MaybeUninit::new(usart);
        (&*stm32::USART1::ptr()).cr3.modify(|_, w| w.dmar().enabled());

        let dma2 = &*(stm32::DMA2::ptr());
        let stream = &dma2.st[5];
        stream.ndtr.write(|w| w.ndt().bits(DMA_BUFFER.len() as u16));
        stream.par.write(|w| w.pa().bits(&(&*(stm32::USART1::ptr())).dr as *const _ as u32));
        let m0ar = &stream.m0ar;
        m0ar.write(|w| w.m0a().bits(DMA_BUFFER.as_ptr() as u32));
        #[rustfmt::skip]
        stream.cr.write(|w| {
            w.chsel().bits(4).minc().incremented().dir().peripheral_to_memory().circ().enabled()
                .pl().very_high().htie().enabled().tcie().enabled().en().enabled()
        });
    }

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM5);
    unsafe {
        cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA2_STREAM5);
    }

    let device = match config {
        SerialConfig::GNSS(gnss) => Device::GNSS(GNSS::new(gnss.protocol)),
        _ => Device::None,
    };
    unsafe {
        DEVICE = device;
        &mut DEVICE
    }
}
