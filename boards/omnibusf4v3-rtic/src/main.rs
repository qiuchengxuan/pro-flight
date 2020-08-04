#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate chips;
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate rs_flight;
extern crate rtic;
extern crate stm32f4xx_hal;

use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::panic::PanicInfo;

use alloc_cortex_m::CortexMHeap;
use cortex_m::peripheral::DWT;
use rtic::cyccnt::U32Ext;
use stm32f4xx_hal::gpio::gpiob;
use stm32f4xx_hal::gpio::{Output, PushPull};
use stm32f4xx_hal::{prelude::*, stm32};

use chips::stm32f4::dfu::Dfu;
use chips::stm32f4::valid_memory_address;

use rs_flight::components::cli::memory;
use rs_flight::components::panic::{log_panic, PanicLogger};
use rs_flight::logger::{self, Level};
use rs_flight::sys::timer;

const LOG_BUFFER_SIZE: usize = 1024;
const SYS_CLOCK: u32 = 128_000_100;
const LED_DURATION: u32 = SYS_CLOCK / 10;

#[link_section = ".uninit.STACKS"]
#[link_section = ".ccmram"]
static mut CCM_MEMORY: [u8; 65536] = [0u8; 65536];

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

macro_rules! panic_logger {
    () => {
        &mut *(&mut CCM_MEMORY[LOG_BUFFER_SIZE] as *mut _ as *mut PanicLogger)
    };
}

#[link_section = ".uninit.STACKS"]
static mut DFU: MaybeUninit<Dfu> = MaybeUninit::uninit();

#[rtic::app(device = stm32f4xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        led: gpiob::PB5<Output<PushPull>>,
    }

    #[init(schedule = [led])]
    fn init(mut context: init::Context) -> init::LateResources {
        let dfu = unsafe { &mut *DFU.as_mut_ptr() };
        dfu.check();
        dfu.arm();

        unsafe { ALLOCATOR.init(CCM_MEMORY.as_ptr() as usize, CCM_MEMORY.len()) }
        context.core.DCB.enable_trace();
        let rcc = context.device.RCC.constrain();
        rcc.cfgr.use_hse(8.mhz()).sysclk(SYS_CLOCK.hz()).freeze();
        DWT::unlock();
        context.core.DWT.enable_cycle_counter();

        unsafe {
            let rcc = &*stm32::RCC::ptr();
            rcc.apb2enr.write(|w| w.syscfgen().enabled());
            rcc.ahb1enr.modify(|_, w| w.dma1en().enabled().dma2en().enabled().crcen().enabled());
        }

        logger::init(Level::Debug);
        memory::init(valid_memory_address);

        let gpio_b = context.device.GPIOB.split();
        let led = gpio_b.pb5.into_push_pull_output();
        context.schedule.led(context.start + LED_DURATION.cycles()).unwrap();
        init::LateResources { led }
    }

    #[task(schedule = [led], resources = [led])]
    fn led(context: led::Context) {
        context.resources.led.toggle().ok();
        context.schedule.led(context.scheduled + LED_DURATION.cycles()).unwrap()
    }

    extern "C" {
        fn EXTI0();
    }
};

#[panic_handler]
unsafe fn panic(panic_info: &PanicInfo) -> ! {
    log_panic(format_args!("{}", panic_info), panic_logger!());
    cortex_m::peripheral::SCB::sys_reset();
}

#[alloc_error_handler]
unsafe fn oom(_: Layout) -> ! {
    log_panic(format_args!("OOM"), panic_logger!());
    cortex_m::peripheral::SCB::sys_reset();
}
