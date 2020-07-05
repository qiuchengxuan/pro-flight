use core::ptr::{read_volatile, write_volatile};

use cortex_m;
use stm32f4xx_hal::{prelude::*, stm32};

const DFU_SAFE: usize = 0xCAFEFEED;
const DFU_MAGIC: usize = 0xDEADBEEF;

pub struct Dfu(usize);

impl Dfu {
    pub fn new() -> Self {
        Self(DFU_SAFE)
    }

    fn enter(&self) {
        cortex_m::Peripherals::take().unwrap();
        let peripherals = stm32::Peripherals::take().unwrap();
        let rcc = peripherals.RCC.constrain();
        rcc.cfgr.sysclk(48.mhz()).freeze();
        unsafe {
            peripherals.SYSCFG.memrm.write(|w| w.bits(1)); // from system memory
            #[cfg(all(cortex_m, feature = "inline-asm"))]
            asm!("eor r0, r0
                  ldr sp, [r0, #0]
                  ldr r0, [r0, #4]
                  bx r0" :::: "volatile");
        }
    }

    pub fn check(&mut self) {
        if unsafe { read_volatile(&self.0) } == DFU_MAGIC {
            unsafe { write_volatile(&mut self.0, DFU_SAFE) };
            self.enter();
        }
    }

    pub fn reboot_into(&mut self) {
        unsafe { write_volatile(&mut self.0, DFU_MAGIC) };
        cortex_m::peripheral::SCB::sys_reset();
    }
}
