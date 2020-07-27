use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

use ascii_osd_hud::telemetry::TelemetrySource;

use bmp280::bus::{DelayNs, DummyOutputPin, SpiBus};
use bmp280::registers::Register;

use embedded_hal::digital::v2::OutputPin;
use max7456::not_null_writer::NotNullWriter;
use max7456::SPI_MODE;
use rs_flight::components::ascii_hud::AsciiHud;
use rs_flight::config;
use rs_flight::datastructures::schedule::Schedulable;
use rs_flight::datastructures::Ratio;
use rs_flight::drivers::bmp280::{init as bmp280_init, on_dma_receive};
use rs_flight::drivers::max7456::init as max7456_init;
use rs_flight::drivers::shared_spi::{SharedSpi, VirtualSpi};
use stm32f4xx_hal::gpio::gpioa::PA15;
use stm32f4xx_hal::gpio::gpiob::PB3;
use stm32f4xx_hal::gpio::gpioc::{PC10, PC11, PC12};
use stm32f4xx_hal::gpio::Input;
use stm32f4xx_hal::gpio::{Floating, Output, PushPull};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Spi};
use stm32f4xx_hal::{prelude::*, stm32};

// NOTE: actually empty, no need to initialize
static mut CS_BARO: MaybeUninit<PB3<Output<PushPull>>> = MaybeUninit::uninit();
static mut CS_OSD: MaybeUninit<PA15<Output<PushPull>>> = MaybeUninit::uninit();
static mut DMA_BUFFER: [u8; 8] = [0u8; 8];
static mut BARO_DMA_FINISHED: AtomicBool = AtomicBool::new(false);

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

#[interrupt]
unsafe fn DMA1_STREAM2() {
    cortex_m::interrupt::free(|_| {
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA1_STREAM2);
        let dma1 = &*stm32::DMA1::ptr();
        dma1.hifcr.write(|w| w.bits(0x3D << 22)); // stream 7
        dma1.lifcr.write(|w| w.bits(0x3D << 16)); // stream 2
    });

    on_dma_receive(&DMA_BUFFER);
    BARO_DMA_FINISHED.store(true, Ordering::Relaxed);
}

pub struct BaroScheduler;

impl Schedulable for BaroScheduler {
    fn schedule(&mut self) {
        unsafe {
            BARO_DMA_FINISHED.store(false, Ordering::Relaxed);
            { &mut *CS_OSD.as_mut_ptr() }.set_high().ok();
            { &mut *CS_BARO.as_mut_ptr() }.set_low().ok();
            let spi3 = &(*stm32::SPI3::ptr());
            let data_register = &spi3.dr as *const _ as u32;
            let dma1 = &*(stm32::DMA1::ptr());
            dma1.hifcr.write(|w| w.bits(0x3D << 22)); // stream 7
            dma1.lifcr.write(|w| w.bits(0x3D << 16)); // stream 2
            let stream = &dma1.st[2]; // dma1 channel 0 stream 2 rx
            let dma_buffer = &DMA_BUFFER;
            stream.ndtr.write(|w| w.ndt().bits(dma_buffer.len() as u16));
            stream.par.write(|w| w.pa().bits(data_register));
            let m0ar = &stream.m0ar;
            m0ar.write(|w| w.m0a().bits(dma_buffer.as_ptr() as u32));
            #[rustfmt::skip]
            stream.cr.write(|w| {
                w.chsel().bits(0).minc().incremented().dir().peripheral_to_memory()
                    .tcie().enabled().en().enabled()
            });
            static READ_REG: [u8; 1] = [Register::PressureMsb as u8 | 0x80];
            let read_reg = &READ_REG;

            // dma1 channel 0 stream 7 tx
            let stream = &dma1.st[7];
            stream.ndtr.write(|w| w.ndt().bits(dma_buffer.len() as u16));
            stream.par.write(|w| w.pa().bits(data_register));
            stream.m0ar.write(|w| w.m0a().bits(read_reg.as_ptr() as u32));
            let cr = &stream.cr;
            cr.write(|w| w.chsel().bits(0).dir().memory_to_peripheral().en().enabled());
        }
    }
}

pub struct OSDScheduler<'a>(AsciiHud<'a>);

impl<'a> OSDScheduler<'a> {
    fn dma_transfer(&self, buffer: &[u8]) {
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
}

impl<'a> Schedulable for OSDScheduler<'a> {
    fn schedule(&mut self) {
        let screen = self.0.draw();
        unsafe {
            while !BARO_DMA_FINISHED.load(Ordering::Relaxed) {}
            { &mut *CS_BARO.as_mut_ptr() }.set_high().ok();
            { &mut *CS_OSD.as_mut_ptr() }.set_low().ok();
        }

        static mut OSD_DMA_BUFFER: [u8; 800] = [0u8; 800];
        let mut dma_buffer = unsafe { OSD_DMA_BUFFER };
        let mut writer = NotNullWriter::new(screen, Default::default());
        let display = writer.write(&mut dma_buffer);
        self.dma_transfer(&display.0);
    }
}

pub fn init<'a>(
    spi3: stm32::SPI3,
    spi3_pins: (PC10<Input<Floating>>, PC11<Input<Floating>>, PC12<Input<Floating>>),
    pa15: PA15<Input<Floating>>,
    pb3: PB3<Input<Floating>>,
    clocks: Clocks,
    telemetry_source: &'static dyn TelemetrySource,
) -> Result<(impl Schedulable, impl Schedulable), Error> {
    let cs_osd = pa15.into_push_pull_output();
    let cs_baro = pb3.into_push_pull_output();
    let (pc10, pc11, pc12) = spi3_pins;
    let sclk = pc10.into_alternate_af6();
    let miso = pc11.into_alternate_af6();
    let mosi = pc12.into_alternate_af6();
    let freq: stm32f4xx_hal::time::Hertz = 10.mhz().into();
    let spi3 = Spi::spi3(spi3, (sclk, miso, mosi), SPI_MODE, freq, clocks);
    let spi = SharedSpi::new(spi3, (cs_baro, cs_osd));

    let mut dummy_cs = DummyOutputPin {};
    let bus = SpiBus::new(VirtualSpi::new(&spi, 0), &mut dummy_cs, TickDelay(clocks.sysclk().0));
    if !bmp280_init(bus).is_ok() {
        warn!("BMP280 init not ok")
    }
    max7456_init(VirtualSpi::new(&spi, 1))?;

    let spi3 = unsafe { &(*stm32::SPI3::ptr()) };
    spi3.cr2.modify(|_, w| w.txdmaen().enabled().rxdmaen().enabled());
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA1_STREAM2);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA1_STREAM2) }
    let config = &config::get().osd;
    Ok((
        BaroScheduler,
        OSDScheduler(AsciiHud::new(
            telemetry_source,
            config.fov,
            Ratio(12, 18).into(),
            config.aspect_ratio.into(),
        )),
    ))
}
