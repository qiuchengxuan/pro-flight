use max7456::{MAX7456, SPI_MODE};
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::gpioc::{PC10, PC11, PC12};
use stm32f4xx_hal::gpio::{Alternate, AF6};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Spi};
use stm32f4xx_hal::timer::{Event, Timer};
use stm32f4xx_hal::{prelude::*, stm32};

use components::max7456_ascii_hud::{Max7456AsciiHud, StubTelemetrySource};

static mut G_SOURCE: StubTelemetrySource = StubTelemetrySource {};
static mut G_TIM4: Option<Timer<stm32::TIM4>> = None;
static mut G_OSD: Option<Max7456AsciiHud<Spi<stm32::SPI3, Spi3Pins>>> = None;

#[interrupt]
fn TIM4() {
    cortex_m::interrupt::free(|_cs| unsafe {
        if let Some(ref mut tim) = G_TIM4 {
            tim.clear_interrupt(Event::TimeOut);
        };
        if let Some(ref mut osd) = G_OSD {
            osd.start_draw();
        }
    });
}

type Spi3Pins = (
    PC10<Alternate<AF6>>,
    PC11<Alternate<AF6>>,
    PC12<Alternate<AF6>>,
);

fn dma1_spi3_stream7_transfer(buffer: &[u8]) {
    let dma1 = unsafe { &*(stm32::DMA1::ptr()) };
    dma1.hifcr.write(|w| {
        w.ctcif7()
            .set_bit()
            .chtif7()
            .set_bit()
            .cteif7()
            .set_bit()
            .cfeif7()
            .set_bit()
    });
    let stream = &dma1.st[7];
    stream.ndtr.write(|w| w.ndt().bits(buffer.len() as u16));
    let spi3 = unsafe { &(*stm32::SPI3::ptr()) };
    spi3.cr2.modify(|_, w| w.txdmaen().enabled());
    let spi3_address = &spi3.dr as *const _ as u32;
    stream.par.write(|w| w.pa().bits(spi3_address));
    let address = buffer.as_ptr() as u32;
    stream.m0ar.write(|w| w.m0a().bits(address));
    stream.cr.write(|w| {
        w.chsel()
            .bits(0)
            .minc()
            .incremented()
            .dir()
            .memory_to_peripheral()
            .en()
            .enabled()
    });
}

pub fn init<'a>(
    spi3: stm32::SPI3,
    pins: Spi3Pins,
    tim4: stm32::TIM4,
    clocks: Clocks,
    delay: &mut Delay,
) -> Result<(), Error> {
    let freq: stm32f4xx_hal::time::Hertz = 10.mhz().into();
    let spi3 = Spi::spi3(spi3, pins, SPI_MODE, freq, clocks);
    let max7456 = MAX7456::new(spi3);
    let mut osd = Max7456AsciiHud::new(unsafe { &G_SOURCE }, max7456, dma1_spi3_stream7_transfer);
    osd.init(delay)?;
    unsafe { G_OSD = Some(osd) };

    let mut timer = Timer::tim4(tim4, 25.hz(), clocks);
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::TIM4);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::TIM4) }
    timer.listen(Event::TimeOut);
    cortex_m::interrupt::free(|_cs| unsafe { G_TIM4 = Some(timer) });
    Ok(())
}
