use stm32f4xx_hal::pac;

pub fn enter() {
    cortex_m::interrupt::disable();
    unsafe {
        (&*pac::SYSCFG::ptr()).memrm.modify(|_, w| w.mem_mode().bits(1));
        cortex_m::asm::bootload(0 as *const u32)
    }
}
