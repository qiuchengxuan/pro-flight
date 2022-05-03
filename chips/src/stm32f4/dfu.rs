use stm32f4xx_hal::pac;

pub fn enter() {
    cortex_m::interrupt::disable();
    unsafe {
        (&*pac::SYSCFG::ptr()).memrm.modify(|_, w| w.mem_mode().bits(1));
        core::arch::asm!("eor r0, r0", "ldr sp, [r0, #0]", "ldr r0, [r0, #4]", "bx r0");
    }
}
