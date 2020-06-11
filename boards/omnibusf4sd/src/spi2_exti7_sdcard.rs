use core::fmt::Write;
use core::mem::MaybeUninit;

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::MODE_3;
use embedded_sdmmc::{Controller, Directory, SdMmcSpi, TimeSource, Timestamp, Volume, VolumeIdx};
use rs_flight::components::logger::Logger;
use stm32f4xx_hal::gpio::gpiob::{PB12, PB13, PB14, PB15, PB7};
use stm32f4xx_hal::gpio::ExtiPin;
use stm32f4xx_hal::gpio::{Floating, Input, PullUp};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Pins, Spi};
use stm32f4xx_hal::stm32::SPI2;
use stm32f4xx_hal::{prelude::*, stm32};

pub struct StubRTC {}

impl TimeSource for StubRTC {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp::from_calendar(2020, 06, 21, 0, 0, 0).ok().unwrap()
    }
}

static mut INT: MaybeUninit<PB7<Input<PullUp>>> = MaybeUninit::uninit();

#[interrupt]
unsafe fn EXTI9_5() {
    let pin = { &mut *INT.as_mut_ptr() };
    cortex_m::interrupt::free(|_| {
        { &mut *INT.as_mut_ptr() }.clear_interrupt_pending_bit();
        pin.clear_interrupt_pending_bit();
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI9_5);
    });

    if pin.is_low().ok().unwrap() {
        log!("SD CARD INSERTED");
    } else {
        log!("SD CARD EJECTED");
    }
}

pub fn init(
    spi2: stm32::SPI2,
    spi2_pins: (PB13<Input<Floating>>, PB14<Input<Floating>>, PB15<Input<Floating>>),
    pb12: PB12<Input<Floating>>,
    clocks: Clocks,
    int: PB7<Input<PullUp>>,
) -> Option<(
    Controller<SdMmcSpi<Spi<SPI2, impl Pins<SPI2>>, impl OutputPin>, StubRTC>,
    Volume,
    Directory,
)> {
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI9_5);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::EXTI9_5) }
    unsafe { INT = MaybeUninit::new(int) };
    let cs = pb12.into_push_pull_output();
    let (pb13, pb14, pb15) = spi2_pins;
    let sclk = pb13.into_alternate_af5();
    let miso = pb14.into_alternate_af5();
    let mosi = pb15.into_alternate_af5();
    let freq: stm32f4xx_hal::time::Hertz = 500.khz().into();
    let spi2 = Spi::spi2(spi2, (sclk, miso, mosi), MODE_3, freq, clocks);
    let stub_rtc = StubRTC {};
    let mut controller = Controller::new(SdMmcSpi::new(spi2, cs), stub_rtc);
    match controller.device().init() {
        Ok(_) => (),
        Err(e) => {
            log!("{:?}", e);
            return None;
        }
    };
    match controller.device().card_size_bytes() {
        Ok(size) => {
            log!("SDCARD size {}GB", size / 1000_000_000);
        }
        Err(e) => {
            log!("Unknown SDCARD size: {:?}", e);
            return None;
        }
    };

    let volumn = match controller.get_volume(VolumeIdx(0)) {
        Ok(v) => {
            log!("Recognized volume 0");
            v
        }
        Err(e) => {
            log!("Get volumn 0 err: {:?}", e);
            return None;
        }
    };
    let root = controller.open_root_dir(&volumn).ok().unwrap();
    Some((controller, volumn, root))
}
