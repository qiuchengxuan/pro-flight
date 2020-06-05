use core::convert::Infallible;
use core::mem::MaybeUninit;

use ascii_osd_hud::telemetry::TelemetrySource;
use bmp280::bus::{DelayNs, SpiBus};
use bmp280::measurement::{Calibration, RawPressure, RawTemperature};
use bmp280::registers::Register;
use embedded_hal::digital::v2::OutputPin;
use max7456::SPI_MODE;
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::gpioa::PA15;
use stm32f4xx_hal::gpio::gpiob::PB3;
use stm32f4xx_hal::gpio::gpioc::{PC10, PC11, PC12};
use stm32f4xx_hal::gpio::{Alternate, AF6};
use stm32f4xx_hal::gpio::{Output, PushPull};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Spi};
use stm32f4xx_hal::timer::{Event, Timer};
use stm32f4xx_hal::{prelude::*, stm32};

use rs_flight::components::ascii_hud::AsciiHud;
use rs_flight::datastructures::event::{event_nop_handler, EventHandler};
use rs_flight::drivers::bmp280::init as bmp280_init;
use rs_flight::drivers::max7456::{init as max7456_init, process_screen};
use rs_flight::drivers::shared_spi::{SharedSpi, VirtualChipSelect, VirtualSpi};
use rs_flight::hal::sensors::Pressure;

static mut TIM7: MaybeUninit<Timer<stm32::TIM7>> = MaybeUninit::uninit();
#[link_section = ".ccmram"]
static mut OSD: MaybeUninit<AsciiHud> = MaybeUninit::uninit();
static mut CS_OSD: MaybeUninit<PA15<Output<PushPull>>> = MaybeUninit::uninit();
static mut CS_BARO: MaybeUninit<PB3<Output<PushPull>>> = MaybeUninit::uninit();
static mut CALIBRATION: MaybeUninit<Calibration> = MaybeUninit::uninit();
static mut DMA_BUFFER: [u8; 8] = [0u8; 8];
static mut BARO_HANDLER: EventHandler<Pressure> = event_nop_handler;

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

type Spi3Pins = (PC10<Alternate<AF6>>, PC11<Alternate<AF6>>, PC12<Alternate<AF6>>);

fn dma1_stream7_transfer_spi3(buffer: &[u8]) {
    let dma1 = unsafe { &*(stm32::DMA1::ptr()) };
    let stream = &dma1.st[7];
    stream.ndtr.write(|w| w.ndt().bits(buffer.len() as u16));
    let spi3 = unsafe { &(*stm32::SPI3::ptr()) };
    let data_register = &spi3.dr as *const _ as u32;
    stream.par.write(|w| w.pa().bits(data_register));
    stream.m0ar.write(|w| w.m0a().bits(buffer.as_ptr() as u32));
    stream.cr.write(|w| {
        w.chsel().bits(0).minc().incremented().dir().memory_to_peripheral().en().enabled()
    });
}

#[interrupt]
unsafe fn DMA1_STREAM2() {
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA1_STREAM2);
    let dma1 = &*stm32::DMA1::ptr();
    dma1.hifcr.write(|w| w.bits(0x3D << 22)); // stream 7
    dma1.lifcr.write(|w| w.bits(0x3D << 16)); // stream 2

    let calibration = &*CALIBRATION.as_ptr();
    let raw_pressure = RawPressure::from_bytes(&DMA_BUFFER[2..]);
    let t_fine = RawTemperature::from_bytes(&DMA_BUFFER[5..]).t_fine(calibration);
    let pressure = raw_pressure.compensated(t_fine, calibration);
    BARO_HANDLER(Pressure(pressure));

    { &mut *CS_BARO.as_mut_ptr() }.set_high().ok();
    { &mut *CS_OSD.as_mut_ptr() }.set_low().ok();
    (&mut *OSD.as_mut_ptr()).start_draw(|screen| {
        process_screen(screen, dma1_stream7_transfer_spi3);
    });
}

#[interrupt]
unsafe fn TIM7() {
    (&mut *TIM7.as_mut_ptr()).clear_interrupt(Event::TimeOut);

    { &mut *CS_OSD.as_mut_ptr() }.set_high().ok();
    { &mut *CS_BARO.as_mut_ptr() }.set_low().ok();

    let spi3 = &(*stm32::SPI3::ptr());
    let data_register = &spi3.dr as *const _ as u32;
    let dma1 = &*(stm32::DMA1::ptr());
    dma1.hifcr.write(|w| w.bits(0x3D << 22)); // stream 7
    dma1.lifcr.write(|w| w.bits(0x3D << 16)); // stream 2
                                              // dma1 channel 0 stream 2 rx
    let stream = &dma1.st[2];
    stream.ndtr.write(|w| w.ndt().bits(DMA_BUFFER.len() as u16));
    stream.par.write(|w| w.pa().bits(data_register));
    let m0ar = &stream.m0ar;
    m0ar.write(|w| w.m0a().bits(DMA_BUFFER.as_ptr() as u32));
    stream.cr.write(|w| {
        w.chsel()
            .bits(0)
            .minc()
            .incremented()
            .dir()
            .peripheral_to_memory()
            .tcie()
            .enabled()
            .en()
            .enabled()
    });

    static READ_REG: [u8; 1] = [Register::PressureMsb as u8 | 0x80];

    // dma1 channel 0 stream 7 tx
    let stream = &dma1.st[7];
    stream.ndtr.write(|w| w.ndt().bits(DMA_BUFFER.len() as u16));
    stream.par.write(|w| w.pa().bits(data_register));
    stream.m0ar.write(|w| w.m0a().bits(READ_REG.as_ptr() as u32));
    let cr = &stream.cr;
    cr.write(|w| w.chsel().bits(0).dir().memory_to_peripheral().en().enabled());
}

pub fn init<'a>(
    spi3: stm32::SPI3,
    tim7: stm32::TIM7,
    pins: Spi3Pins,
    mut cs_osd: PA15<Output<PushPull>>,
    mut cs_baro: PB3<Output<PushPull>>,
    clocks: Clocks,
    telemetry_source: &'static dyn TelemetrySource,
    baro_handler: EventHandler<Pressure>,
    delay: &mut Delay,
) -> Result<(), Error> {
    let freq: stm32f4xx_hal::time::Hertz = 10.mhz().into();
    let spi3 = Spi::spi3(spi3, pins, SPI_MODE, freq, clocks);
    let mut css: [&mut dyn OutputPin<Error = Infallible>; 2] = [&mut cs_baro, &mut cs_osd];
    let spi = SharedSpi::new(spi3, &mut css);

    let mut virtual_cs = VirtualChipSelect::new(&spi, 0);
    let bus = SpiBus::new(VirtualSpi::new(&spi, 0), &mut virtual_cs, TickDelay(clocks.sysclk().0));
    if let Some(calibration) = bmp280_init(bus, delay).ok().unwrap() {
        unsafe { CALIBRATION = MaybeUninit::new(calibration) };
    } else {
        return Ok(());
    }
    max7456_init(VirtualSpi::new(&spi, 1), delay)?;

    let spi3 = unsafe { &(*stm32::SPI3::ptr()) };
    spi3.cr2.modify(|_, w| w.txdmaen().enabled().rxdmaen().enabled());
    let mut timer = Timer::tim7(tim7, 50.hz(), clocks);
    timer.listen(Event::TimeOut);
    unsafe {
        OSD = MaybeUninit::new(AsciiHud::new(telemetry_source));
        TIM7 = MaybeUninit::new(timer);
        CS_OSD = MaybeUninit::new(cs_osd);
        CS_BARO = MaybeUninit::new(cs_baro);
        BARO_HANDLER = baro_handler;
    }

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA1_STREAM2);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA1_STREAM2) }
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::TIM7);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::TIM7) }
    Ok(())
}
