use core::ptr::{read_volatile, write_volatile};

const DFU_ARM: usize = 0x4446550A;

pub struct Dfu(usize);

impl Dfu {
    pub fn new() -> Self {
        Self(0)
    }

    #[inline(never)]
    fn enter(&self) -> ! {
        cortex_m::interrupt::disable();
        unsafe {
            llvm_asm!("ldr r0, =0x1fff0000
                       ldr sp, [r0, #0]
                       ldr r0, [r0, #4]
                       bx r0" :::: "volatile");
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
        unsafe { write_volatile(&mut self.0, 0) };
    }
}
