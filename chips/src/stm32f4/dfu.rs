use core::ptr::{read_volatile, write_volatile};

const DFU_ARM: usize = 0xDEADBEEF;
const DFU_DISARM: usize = 0xCAFEFEED;

pub struct Dfu(pub usize);

impl Dfu {
    fn enter(&self) -> ! {
        unsafe {
            cortex_m::register::msp::write(0x1FFF0004);
            let reset_handler: fn() = core::mem::transmute(0x1FFF0000);
            reset_handler();
        }
        loop {}
    }

    pub fn check(&mut self) {
        if unsafe { read_volatile(&self.0) } == DFU_ARM {
            self.disarm();
            self.enter();
        } else {
            self.arm()
        }
    }

    pub fn arm(&mut self) {
        unsafe { write_volatile(&mut self.0, DFU_ARM) };
    }

    pub fn disarm(&mut self) {
        unsafe { write_volatile(&mut self.0, DFU_DISARM) };
    }
}
