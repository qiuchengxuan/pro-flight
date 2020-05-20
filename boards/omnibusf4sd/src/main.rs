#![no_main]
#![no_std]

#[macro_use]
extern crate cortex_m_rt;
extern crate btoi;
extern crate cast;
extern crate chips;
extern crate components;
extern crate cortex_m;
extern crate cortex_m_systick_countdown;
extern crate max7456;
extern crate nb;
extern crate panic_semihosting;
extern crate stm32f4xx_hal;
extern crate usb_device;

mod console;
mod max7456_spi3;
mod mpu6000_spi1;
mod usb_serial;

use core::sync::atomic::{AtomicBool, Ordering};

use arrayvec::ArrayVec;
use btoi::btoi_radix;
use cortex_m_rt::ExceptionFrame;
use cortex_m_systick_countdown::{MillisCountDown, PollingSysTick, SysTickCalibration};
use numtoa::NumToA;
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use stm32f4xx_hal::timer::{Event, Timer};
use stm32f4xx_hal::{prelude::*, stm32};

use chips::stm32f4::dfu::Dfu;
use components::max7456_ascii_hud::{Max7456AsciiHud, StubTelemetrySource};
use components::sysled::Sysled;

use console::Console;

static mut G_TIM4: Option<Timer<stm32::TIM4>> = None;
static G_OSD_HUD_REFRESH: AtomicBool = AtomicBool::new(false);

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

#[interrupt]
fn TIM4() {
    cortex_m::interrupt::free(|_cs| unsafe {
        if let Some(ref mut tim) = G_TIM4 {
            tim.clear_interrupt(Event::TimeOut);
        };
    });
    G_OSD_HUD_REFRESH.store(true, Ordering::Relaxed);
}

#[entry]
fn main() -> ! {
    let mut dfu = Dfu::new();
    dfu.check();

    let cortex_m_peripherals = cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(64.mhz())
        .require_pll48clk()
        .freeze();

    let mut delay = Delay::new(cortex_m_peripherals.SYST, clocks);

    let gpio_a = peripherals.GPIOA.split();
    let gpio_b = peripherals.GPIOB.split();
    let gpio_c = peripherals.GPIOC.split();

    unsafe { &(*stm32::RCC::ptr()) }
        .ahb1enr
        .modify(|_, w| w.dma1en().enabled());
    let dma1 = &peripherals.DMA1;
    let dma_transfer = |buffer: &[u8]| {
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
    };

    let _cs = gpio_a.pa15.into_push_pull_output();
    let sclk = gpio_c.pc10.into_alternate_af6();
    let miso = gpio_c.pc11.into_alternate_af6();
    let mosi = gpio_c.pc12.into_alternate_af6();
    let result = max7456_spi3::init(peripherals.SPI3, (sclk, miso, mosi), clocks);
    let source = StubTelemetrySource {};
    let mut osd = Max7456AsciiHud::new(&source, result.unwrap(), dma_transfer);
    osd.init(&mut delay).ok();

    let cs = gpio_a.pa4.into_push_pull_output();
    let sclk = gpio_a.pa5.into_alternate_af5();
    let miso = gpio_a.pa6.into_alternate_af5();
    let mosi = gpio_a.pa7.into_alternate_af5();
    let result = mpu6000_spi1::init(peripherals.SPI1, (sclk, miso, mosi), cs, clocks, &mut delay);

    let calibration = SysTickCalibration::from_clock_hz(clocks.sysclk().0);
    let systick = PollingSysTick::new(delay.free(), &calibration);

    let pin = gpio_b.pb5.into_push_pull_output();
    let mut sysled = Sysled::new(pin, MillisCountDown::new(&systick));

    let mut timer = Timer::tim4(peripherals.TIM4, 25.hz(), clocks);
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::TIM4);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::TIM4) }
    timer.listen(Event::TimeOut);
    cortex_m::interrupt::free(|_cs| unsafe { G_TIM4 = Some(timer) });

    let usb = USB {
        usb_global: peripherals.OTG_FS_GLOBAL,
        usb_device: peripherals.OTG_FS_DEVICE,
        usb_pwrclk: peripherals.OTG_FS_PWRCLK,
        pin_dm: gpio_a.pa11.into_alternate_af10(),
        pin_dp: gpio_a.pa12.into_alternate_af10(),
    };

    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });
    let mut usb_serial = usb_serial::USBSerial::new(&usb_bus);
    let console = Console::new(&mut usb_serial);

    let mut vec = ArrayVec::<[u8; 80]>::new();
    loop {
        sysled.check_toggle().unwrap();
        if G_OSD_HUD_REFRESH.swap(false, Ordering::Relaxed) {
            osd.start_draw();
        }

        let option = console.try_read_line(&mut vec);
        if option.is_none() {
            continue;
        }
        let line = option.unwrap();
        if line.len() > 0 {
            if line == *b"dfu" {
                dfu.reboot_into();
            } else if line == *b"reboot" {
                cortex_m::peripheral::SCB::sys_reset();
            } else if line == *b"check" {
                match result {
                    Ok(b) => {
                        if b {
                            console.write(b"found mpu6000\r\n");
                        } else {
                            console.write(b"not mpu6000\r\n");
                        }
                    }
                    Err(_) => {
                        console.write(b"spi1 error");
                    }
                }
            } else if line.starts_with(b"read") {
                let mut iter = line.split(|b| *b == ' ' as u8);
                iter.next();
                let address = if let Some(address) = iter.next() {
                    match btoi_radix::<u32>(address, 16) {
                        Ok(address) => address,
                        _ => 0,
                    }
                } else {
                    0
                };
                if 0x40000000 <= address && address <= 0xA0000FFF {
                    let value = unsafe { *(address as *const u32) };
                    let mut buffer = [0u8; 10];
                    console.write(b"Result: ");
                    console.write(value.numtoa(16, &mut buffer));
                    console.write(b"\r\n")
                }
            } else if line.starts_with(b"write") {
                let mut iter = line.split(|b| *b == ' ' as u8);
                iter.next();
                let mut address = if let Some(address) = iter.next() {
                    match btoi_radix::<u32>(address, 16) {
                        Ok(address) => address,
                        _ => 0,
                    }
                } else {
                    0
                };
                let value = if let Some(value) = iter.next() {
                    match btoi_radix::<u32>(value, 16) {
                        Ok(value) => value,
                        _ => {
                            address = 0;
                            0
                        }
                    }
                } else {
                    address = 0;
                    0
                };
                if 0x40000000 <= address && address <= 0xA0000FFF {
                    unsafe { *(address as *mut u32) = value };
                    let mut count_down = MillisCountDown::new(&systick);
                    count_down.start_ms(50);
                    nb::block!(count_down.wait()).unwrap();
                    let value = unsafe { *(address as *const u32) };
                    console.write(b"Write result: ");
                    let mut buffer = [0u8; 10];
                    console.write(value.numtoa(16, &mut buffer));
                    console.write(b"\r\n");
                } else {
                    console.write(b"Bad input\r\n");
                }
            } else {
                console.write(b"unknown input: ");
                console.write(line);
                console.write(b"\r\n")
            }
        }
        console.write(b"# ");
        vec.clear();
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
