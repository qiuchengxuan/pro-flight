#![no_main]
#![no_std]
#![feature(llvm_asm)]

#[macro_use]
extern crate cortex_m_rt;
extern crate btoi;
extern crate cast;
extern crate cortex_m;
extern crate nb;
extern crate panic_semihosting;
extern crate stm32f4xx_hal;
extern crate usb_device;

mod console;
mod max7456_spi3;
mod mpu6000_spi1;
mod usb_serial;

use core::mem::MaybeUninit;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering};

use arrayvec::ArrayVec;
use btoi::btoi_radix;
use cortex_m_rt::ExceptionFrame;
use numtoa::NumToA;
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use stm32f4xx_hal::timer::{Event, Timer};
use stm32f4xx_hal::{prelude::*, stm32};

use console::Console;

static mut G_TIM4: Option<Timer<stm32::TIM4>> = None;
static G_LED_TOGGLE: AtomicBool = AtomicBool::new(false);

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

const DFU_SAFE: u32 = 0xCAFEFEED;
const DFU_MAGIC: u32 = 0xDEADBEEF;

#[interrupt]
fn TIM4() {
    cortex_m::interrupt::free(|_cs| unsafe {
        if let Some(ref mut tim) = G_TIM4 {
            tim.clear_interrupt(Event::TimeOut);
        };
    });
    G_LED_TOGGLE.store(true, Ordering::Relaxed);
}

fn enter_dfu() {
    cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();
    rcc.cfgr.sysclk(48.mhz()).freeze();
    unsafe {
        peripherals.SYSCFG.memrm.write(|w| w.bits(1)); // from system memory
        llvm_asm!("eor r0, r0
                   ldr sp, [r0, #0]
                   ldr r0, [r0, #4]
                   bx r0" :::: "volatile");
    }
}

#[entry]
fn main() -> ! {
    let mut dfu_flag: u32 = unsafe { MaybeUninit::uninit().assume_init() };
    if unsafe { read_volatile(&dfu_flag) } == DFU_MAGIC {
        unsafe { write_volatile(&mut dfu_flag, DFU_SAFE) };
        enter_dfu();
    }

    let cortex_m_peripherals = cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(64.mhz())
        .require_pll48clk()
        .freeze();

    let gpio_a = peripherals.GPIOA.split();
    let gpio_b = peripherals.GPIOB.split();
    let gpio_c = peripherals.GPIOC.split();

    let mut delay = Delay::new(cortex_m_peripherals.SYST, clocks);

    let cs = gpio_a.pa4.into_push_pull_output();
    let sclk = gpio_a.pa5.into_alternate_af5();
    let miso = gpio_a.pa6.into_alternate_af5();
    let mosi = gpio_a.pa7.into_alternate_af5();
    let result = mpu6000_spi1::init(peripherals.SPI1, (sclk, miso, mosi), cs, clocks, &mut delay);

    let _cs = gpio_a.pa15.into_push_pull_output();
    let sclk = gpio_c.pc10.into_alternate_af6();
    let miso = gpio_c.pc11.into_alternate_af6();
    let mosi = gpio_c.pc12.into_alternate_af6();
    let _ = max7456_spi3::init(peripherals.SPI3, (sclk, miso, mosi), clocks, &mut delay);

    let mut led = gpio_b.pb5.into_push_pull_output();
    led.set_low().unwrap();

    let mut timer = Timer::tim4(peripherals.TIM4, 10.hz(), clocks);
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
        if G_LED_TOGGLE.swap(false, Ordering::Relaxed) {
            led.toggle().unwrap();
        }

        let option = console.try_read_line(&mut vec);
        if option.is_none() {
            continue;
        }
        let line = option.unwrap();
        if line.len() > 0 {
            if line == *b"dfu" {
                unsafe { write_volatile(&mut dfu_flag, DFU_MAGIC) };
                cortex_m::peripheral::SCB::sys_reset();
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
                    delay.delay_ms(50u8);
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
