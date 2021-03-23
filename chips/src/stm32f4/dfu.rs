use stm32f4xx_hal::stm32;

pub fn enter() {
    let peripherals = stm32::Peripherals::take().unwrap();
    unsafe {
        peripherals.SYSCFG.memrm.modify(|_, w| w.mem_mode().bits(1));
        asm!("eor r0, r0", "ldr sp, [r0, #0]", "ldr r0, [r0, #4]", "bx r0");
    }
}
