use core::ptr::{read_volatile, write_volatile};

use stm32f4xx_hal::stm32;

const DFU_ARM: usize = 0xDEADBEEF;

pub struct Dfu(pub usize);

impl Dfu {
    fn enter(&self) -> ! {
        let peripherals = stm32::Peripherals::take().unwrap();
        unsafe {
            peripherals.SYSCFG.memrm.modify(|_, w| w.mem_mode().bits(1));
            cortex_m::interrupt::disable();
            cortex_m::asm::bootstrap(0x1FFF0000 as *const u32, 0x1FFF0004 as *const u32)
        }
    }

    pub fn check(&mut self) {
        if unsafe { read_volatile(&self.0) } == DFU_ARM {
            self.disarm();
            self.enter();
        }
    }

    pub fn arm(&mut self) {
        unsafe { write_volatile(&mut self.0, DFU_ARM) };
    }

    pub fn disarm(&mut self) {
        unsafe { write_volatile(&mut self.0, 0) };
    }
}
