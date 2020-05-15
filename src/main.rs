#![no_main]
#![no_std]
#![feature(llvm_asm)]

#[macro_use]
extern crate cortex_m_rt;
extern crate cast;
extern crate cortex_m;
extern crate panic_semihosting;
extern crate stm32f4xx_hal;
extern crate usb_device;

mod usb_serial;

use core::mem::MaybeUninit;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m_rt::ExceptionFrame;
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use stm32f4xx_hal::timer::{Event, Timer};
use stm32f4xx_hal::{prelude::*, stm32};

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

    cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .require_pll48clk()
        .freeze();

    let gpio_b = peripherals.GPIOB.split();
    let mut led = gpio_b.pb5.into_push_pull_output();
    led.set_low().unwrap();

    let mut timer = Timer::tim4(peripherals.TIM4, 10.hz(), clocks);
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::TIM4);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::TIM4) }
    timer.listen(Event::TimeOut);

    cortex_m::interrupt::free(|_cs| unsafe { G_TIM4 = Some(timer) });

    let gpio_a = peripherals.GPIOA.split();
    let usb = USB {
        usb_global: peripherals.OTG_FS_GLOBAL,
        usb_device: peripherals.OTG_FS_DEVICE,
        usb_pwrclk: peripherals.OTG_FS_PWRCLK,
        pin_dm: gpio_a.pa11.into_alternate_af10(),
        pin_dp: gpio_a.pa12.into_alternate_af10(),
    };

    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });
    let mut usb_serial = usb_serial::USBSerial::new(&usb_bus);

    let mut buf = [0u8; 80];
    let mut offset = 0;
    loop {
        if G_LED_TOGGLE.swap(false, Ordering::Relaxed) {
            led.toggle().unwrap();
        }

        if !usb_serial.poll() {
            continue;
        }

        let input = usb_serial.read(&mut buf[offset..]);
        usb_serial.write(input);
        let input_len = input.len();
        offset += input_len;
        if let Some(eol) = input.iter().position(|&b| b == '\r' as u8) {
            usb_serial.write(b"\r\n");
            let cmd = &buf[..offset - input_len + eol];
            offset = 0;
            if cmd.len() == 0 {
                continue;
            }
            if cmd == *b"dfu" {
                unsafe { write_volatile(&mut dfu_flag, DFU_MAGIC) };
                cortex_m::peripheral::SCB::sys_reset();
            } else {
                usb_serial.write(b"unknown input\r\n");
            }
        }
        if offset >= buf.len() {
            offset = 0;
        }
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
