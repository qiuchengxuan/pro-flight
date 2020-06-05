use core::mem::MaybeUninit;

use ascii_osd_hud::telemetry::TelemetrySource;
use max7456::SPI_MODE;
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::gpioc::{PC10, PC11, PC12};
use stm32f4xx_hal::gpio::{Alternate, AF6};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Spi};
use stm32f4xx_hal::timer::{Event, Timer};
use stm32f4xx_hal::{prelude::*, stm32};

use rs_flight::components::ascii_hud::AsciiHud;
use rs_flight::drivers::max7456::{init as max7456_init, process_screen};

static mut G_TIM7: MaybeUninit<Timer<stm32::TIM7>> = MaybeUninit::uninit();
#[link_section = ".ccmram"]
static mut G_OSD: MaybeUninit<AsciiHud> = MaybeUninit::uninit();

type Spi3Pins = (PC10<Alternate<AF6>>, PC11<Alternate<AF6>>, PC12<Alternate<AF6>>);

fn dma1_stream7_transfer_spi3(buffer: &[u8]) {
    let dma1 = unsafe { &*(stm32::DMA1::ptr()) };
    unsafe { dma1.hifcr.write(|w| w.bits(0x3D << 22)) };

    let stream = &dma1.st[7];
    stream.ndtr.write(|w| w.ndt().bits(buffer.len() as u16));
    let spi3 = unsafe { &(*stm32::SPI3::ptr()) };
    spi3.cr2.modify(|_, w| w.txdmaen().enabled());
    let data_register = &spi3.dr as *const _ as u32;
    stream.par.write(|w| w.pa().bits(data_register));
    stream.m0ar.write(|w| w.m0a().bits(buffer.as_ptr() as u32));
    stream.cr.write(|w| {
        w.chsel().bits(0).minc().incremented().dir().memory_to_peripheral().en().enabled()
    });
}

#[interrupt]
unsafe fn TIM7() {
    let spi3 = &(*stm32::SPI3::ptr());
    spi3.cr2.modify(|_, w| w.txdmaen().disabled());

    (&mut *G_TIM7.as_mut_ptr()).clear_interrupt(Event::TimeOut);
    (&mut *G_OSD.as_mut_ptr()).start_draw(|screen| {
        process_screen(screen, dma1_stream7_transfer_spi3);
    });
}

pub fn init<'a>(
    spi3: stm32::SPI3,
    tim7: stm32::TIM7,
    pins: Spi3Pins,
    clocks: Clocks,
    telemetry_source: &'static dyn TelemetrySource,
    delay: &mut Delay,
) -> Result<(), Error> {
    let freq: stm32f4xx_hal::time::Hertz = 10.mhz().into();
    let spi3 = Spi::spi3(spi3, pins, SPI_MODE, freq, clocks);
    max7456_init(spi3, delay)?;

    let osd = AsciiHud::new(telemetry_source);
    unsafe { G_OSD = MaybeUninit::new(osd) };

    let mut timer = Timer::tim7(tim7, 50.hz(), clocks);
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::TIM7);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::TIM7) }
    timer.listen(Event::TimeOut);
    unsafe { G_TIM7 = MaybeUninit::new(timer) };
    Ok(())
}
