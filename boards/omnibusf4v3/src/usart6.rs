use rs_flight::config::SerialConfig;
use rs_flight::drivers::sbus::SbusReceiver;
use rs_flight::drivers::uart::Device;
use stm32f4xx_hal::gpio::gpioc;
use stm32f4xx_hal::gpio::{Floating, Input};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::serial::config::{Config, StopBits};
use stm32f4xx_hal::serial::Serial;
use stm32f4xx_hal::{prelude::*, stm32};

type PC6 = gpioc::PC6<Input<Floating>>;
type PC7 = gpioc::PC7<Input<Floating>>;

const HTIF_OFFSET: usize = 4;
const STREAM1_OFFSET: usize = 6;

static mut DMA_BUFFER: [u8; 64] = [0u8; 64];
static mut DEVICE: Option<Device> = None;

#[interrupt]
unsafe fn DMA2_STREAM1() {
    let mut half = false;
    cortex_m::interrupt::free(|_| {
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM1);
        let dma2 = &*stm32::DMA2::ptr();
        half = dma2.lisr.read().bits() & (1 << (HTIF_OFFSET + STREAM1_OFFSET)) > 0;
        dma2.lifcr.write(|w| w.bits(0x3D << STREAM1_OFFSET));
    });
    if let Some(ref mut device) = DEVICE {
        device.handle(&DMA_BUFFER, half);
    }
}

pub fn init(
    usart6: stm32::USART6,
    pins: (PC6, PC7),
    nvic: &mut cortex_m::peripheral::NVIC,
    config: &SerialConfig,
    clocks: Clocks,
) -> Option<&'static mut Device> {
    let mut cfg = Config::default();
    match config {
        SerialConfig::SBUS(sbus) => {
            debug!("Config USART6 as SBUS receiver");
            cfg = cfg.baudrate(sbus.baudrate().bps());
            // word-length-9 must be selected when parity is even
            cfg = cfg.stopbits(StopBits::STOP2).parity_even().wordlength_9();
        }
        _ => return None,
    };

    let (pc6, pc7) = pins;
    let pins = (pc6.into_alternate_af8(), pc7.into_alternate_af8());
    Serial::usart6(usart6, pins, cfg, clocks).unwrap();

    // dma2 stream1 channel 5 rx
    unsafe {
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

    let device = match config {
        SerialConfig::SBUS(_) => {
            unsafe { nvic.set_priority(stm32::Interrupt::DMA2_STREAM1, 18) }; // No less than DMA1 stream0
            Device::SBUS(SbusReceiver::new())
        }
        _ => return None,
    };
    unsafe { DEVICE = Some(device) }
    unsafe { DEVICE.as_mut() }
}
