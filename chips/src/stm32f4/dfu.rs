use core::ptr::{read_volatile, write_volatile};

const DFU_SAFE: usize = 0xCAFEFEED;
const DFU_MAGIC: usize = 0xDEADBEEF;

pub struct Dfu(usize);

impl Dfu {
    pub fn new() -> Self {
        Self(DFU_SAFE)
    }

    #[inline(never)]
    fn enter(&self) {
        unsafe {
            llvm_asm!("ldr r0, =0x40023844
                       ldr r1, =0x00004000
                       str r1, [r0, #0]
                       ldr r0, =0x40013800
                       ldr r1, =0x00000001
                       str r1, [r0, #0]
                       ldr r0, =0x1fff0000
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

    pub fn arm(&mut self) {
        unsafe { write_volatile(&mut self.0, DFU_MAGIC) };
    }

    pub fn disarm(&mut self) {
        unsafe { write_volatile(&mut self.0, DFU_SAFE) };
    }
}
