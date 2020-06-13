use core::mem::MaybeUninit;

use embedded_hal::spi::MODE_3;
use embedded_sdmmc::{Controller, SdMmcSpi, TimeSource, Timestamp};
use rs_flight::drivers::sdcard::Sdcard;
use rs_flight::sys::fs::{set_media, Error, FileDescriptor, Media, OpenOptions, Schema};
use stm32f4xx_hal::gpio::gpiob::{PB12, PB13, PB14, PB15, PB7};
use stm32f4xx_hal::gpio::ExtiPin;
use stm32f4xx_hal::gpio::{Alternate, Floating, Input, Output, PullUp, PushPull, AF5};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::Spi;
use stm32f4xx_hal::stm32::SPI2;
use stm32f4xx_hal::{prelude::*, stm32};

pub struct StubRTC {}

impl TimeSource for StubRTC {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp::from_calendar(2020, 06, 21, 0, 0, 0).ok().unwrap()
    }
}

pub type SPI = Spi<SPI2, (PB13<Alternate<AF5>>, PB14<Alternate<AF5>>, PB15<Alternate<AF5>>)>;
pub type CS = PB12<Output<PushPull>>;
static mut CONTROLLER: MaybeUninit<Controller<SdMmcSpi<SPI, CS>, StubRTC>> = MaybeUninit::uninit();
static mut SDCARD: Option<Sdcard<SdMmcSpi<SPI, CS>, StubRTC>> = None;

fn open(path: &str, options: OpenOptions) -> Result<FileDescriptor, Error> {
    unsafe { SDCARD.as_mut().unwrap() }.open(path, options)
}

fn close(fd: FileDescriptor) {
    unsafe { SDCARD.as_mut().unwrap() }.close(fd)
}

fn read(fd: &FileDescriptor, buf: &mut [u8]) -> Result<usize, Error> {
    unsafe { SDCARD.as_mut().unwrap() }.read(fd, buf)
}

fn probe_sdcard() {
    let controller = unsafe { &mut *CONTROLLER.as_mut_ptr() };
    match controller.device().init() {
        Ok(_) => (),
        Err(e) => {
            debug!("{:?}", e);
            return;
        }
    }
    unsafe {
        SDCARD = Sdcard::new(controller);
        if SDCARD.is_some() {
            set_media(Schema::Sdcard, Media { open, close, read });
        }
    }
}

static mut SDCARD_PRESENT_INT: MaybeUninit<PB7<Input<PullUp>>> = MaybeUninit::uninit();

#[interrupt]
unsafe fn EXTI9_5() {
    let pin = { &mut *SDCARD_PRESENT_INT.as_mut_ptr() };
    cortex_m::interrupt::free(|_| {
        pin.clear_interrupt_pending_bit();
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI9_5);
    });

    if pin.is_low().ok().unwrap() {
        debug!("SD CARD INSERTED");
        probe_sdcard();
    } else {
        debug!("SD CARD EJECTED");
        if let Some(mut sdcard) = SDCARD.take() {
            sdcard.destroy();
        }
        set_media(Schema::Sdcard, Media::default());
    }
}

pub type Spi2Pins = (PB13<Input<Floating>>, PB14<Input<Floating>>, PB15<Input<Floating>>);

pub fn init(
    spi2: stm32::SPI2,
    spi2_pins: Spi2Pins,
    pb12: PB12<Input<Floating>>,
    clocks: Clocks,
    int: PB7<Input<PullUp>>,
) {
    let cs = pb12.into_push_pull_output();
    let (pb13, pb14, pb15) = spi2_pins;
    let sclk = pb13.into_alternate_af5();
    let miso = pb14.into_alternate_af5();
    let mosi = pb15.into_alternate_af5();
    let freq: stm32f4xx_hal::time::Hertz = 500.khz().into();
    let spi2 = Spi::spi2(spi2, (sclk, miso, mosi), MODE_3, freq, clocks);
    let stub_rtc = StubRTC {};
    let controller = Controller::new(SdMmcSpi::new(spi2, cs), stub_rtc);
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI9_5);
    unsafe {
        CONTROLLER = MaybeUninit::new(controller);
        SDCARD_PRESENT_INT = MaybeUninit::new(int);
        cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::EXTI9_5);
    }
    probe_sdcard()
}
