use core::mem::MaybeUninit;

use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::gpioc::{PC10, PC11, PC12};
use stm32f4xx_hal::gpio::{Alternate, AF6};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Spi};
use stm32f4xx_hal::timer::{Event, Timer};
use stm32f4xx_hal::{prelude::*, stm32};

use max7456::{MAX7456, SPI_MODE};

use rs_flight::components::max7456_ascii_hud::{self, Max7456AsciiHud, StubTelemetrySource};
use rs_flight::hal::imu::IMU;

static mut G_TIM7: MaybeUninit<Timer<stm32::TIM7>> = MaybeUninit::uninit();
static mut G_OSD: MaybeUninit<Max7456AsciiHud> = MaybeUninit::uninit();

static mut G_SOURCE: MaybeUninit<StubTelemetrySource> = MaybeUninit::uninit();

fn clear_dma1_stream7_tx_interrupts() {
    let dma1 = unsafe { &*(stm32::DMA1::ptr()) };
    dma1.hifcr
        .write(|w| w.ctcif7().set_bit().chtif7().set_bit().cteif7().set_bit().cfeif7().set_bit())
}

#[interrupt]
fn TIM7() {
    cortex_m::interrupt::free(|_cs| unsafe {
        (&mut *G_TIM7.as_mut_ptr()).clear_interrupt(Event::TimeOut);
    });
    unsafe {
        (&mut *G_OSD.as_mut_ptr()).start_draw();
    }
}

type Spi3Pins = (PC10<Alternate<AF6>>, PC11<Alternate<AF6>>, PC12<Alternate<AF6>>);

fn dma1_stream7_transfer_spi3(buffer: &[u8]) {
    clear_dma1_stream7_tx_interrupts();
    let dma1 = unsafe { &*(stm32::DMA1::ptr()) };
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

pub fn init<'a>(
    spi3: stm32::SPI3,
    tim7: stm32::TIM7,
    pins: Spi3Pins,
    clocks: Clocks,
    imu: &'static dyn IMU,
    delay: &mut Delay,
) -> Result<(), Error> {
    let freq: stm32f4xx_hal::time::Hertz = 10.mhz().into();
    let spi3 = Spi::spi3(spi3, pins, SPI_MODE, freq, clocks);
    let mut max7456 = MAX7456::new(spi3);
    max7456_ascii_hud::init(&mut max7456, delay)?;

    unsafe { &(*stm32::RCC::ptr()) }.ahb1enr.modify(|_, w| w.dma1en().enabled());

    unsafe { G_SOURCE = MaybeUninit::new(StubTelemetrySource::new(imu)) };
    let osd = Max7456AsciiHud::new(unsafe { &*G_SOURCE.as_ptr() }, dma1_stream7_transfer_spi3);
    unsafe { G_OSD = MaybeUninit::new(osd) };

    let mut timer = Timer::tim7(tim7, 50.hz(), clocks);
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::TIM7);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::TIM7) }
    timer.listen(Event::TimeOut);
    unsafe { G_TIM7 = MaybeUninit::new(timer) };
    Ok(())
}
