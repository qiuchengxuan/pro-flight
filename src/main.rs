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

use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m_rt::ExceptionFrame;
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use stm32f4xx_hal::timer::{Event, Timer};
use stm32f4xx_hal::{prelude::*, stm32};

static mut G_TIM4: Option<Timer<stm32::TIM4>> = None;
static G_LED_ON: AtomicBool = AtomicBool::new(false);

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

const DFU_SAFE: u32 = 0xCAFEFEED;
const DFU_MAGIC: u32 = 0xDEEDBEEF;
static mut G_DFU: u32 = DFU_SAFE;

#[interrupt]
fn TIM4() {
    cortex_m::interrupt::free(|_cs| unsafe {
        if let Some(ref mut tim) = G_TIM4 {
            tim.clear_interrupt(Event::TimeOut);
        };
    });
    let led_on = G_LED_ON.load(Ordering::Relaxed);
    G_LED_ON.store(!led_on, Ordering::Relaxed);
}

fn enter_dfu() {
    unsafe { G_DFU = DF_SAFE }
    let peripherals = stm32::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();
    rcc.cfgr.sysclk(48.mhz()).freeze();
    unsafe {
        peripherals.SYSCFG.memrm.write(|w| w.bits(1)); // from system memory
        cortex_m::register::msp::write(0x2001BFFF);
        llvm_asm!("ldr r0, 0x4
                   bx r0"
                  :
                  :
                  :
                  : "volatile")
    }
}

#[allow(unused_must_use)]
#[entry]
fn main() -> ! {
    if unsafe { G_DFU == DFU_MAGIC } {
        enter_dfu();
    }

    let cortex_m_peripherals = cortex_m::Peripherals::take().unwrap();
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
    led.set_low();

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

    let mut delay = Delay::new(cortex_m_peripherals.SYST, clocks);

    let mut buf = [0u8; 80];
    let mut offset = 0;
    loop {
        let led_on = G_LED_ON.load(Ordering::Relaxed);
        if led_on {
            led.set_low();
            delay.delay_ms(10_u16);
            led.set_high();
            delay.delay_ms(10_u16);
            led.set_low();
        } else {
            led.set_high();
        }

        if !usb_serial.poll() {
            continue;
        }

        let input = usb_serial.read(&mut buf[offset..]);
        usb_serial.write(input);
        let input_len = input.len();
        offset += input_len;
        match input.iter().position(|&b| b == '\r' as u8) {
            Some(eol) => {
                if buf[..offset - input_len + eol] == *b"dfu" {
                    unsafe { G_DFU = DFU_MAGIC }
                    cortex_m::peripheral::SCB::sys_reset();
                } else {
                    usb_serial.write(b"\r\nunknown input\r\n")
                }
                offset = 0;
            }
            _ => {}
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
