use drone_core::fib::Yielded;
use drone_cortexm::reg::field::WRwRegFieldBitAtomic;
use drone_cortexm::thr::ThrNvic;
use drone_cortexm::{reg::prelude::*, thr::prelude::*};
use drone_stm32_map::periph::exti::{
    ExtiMap, ExtiPeriph, ExtiPrPif, ExtiSwierSwi, ExtiSwierSwiOpt,
};
use hal::event::Notifier;

pub struct SoftIntNotifier<T>(T);

impl<T: WRwRegFieldBitAtomic<Srt>> Notifier for SoftIntNotifier<T>
where
    T::Reg: RReg<Srt> + WReg<Srt>,
{
    fn notify(&mut self) {
        self.0.set_bit()
    }
}

pub fn make_trigger<T, M>(thread: T, regs: ExtiPeriph<M>) -> impl Notifier
where
    T: ThrNvic,
    M: ExtiMap + ExtiPrPif + ExtiSwierSwiOpt + ExtiSwierSwi,
{
    regs.exti_imr_im.set_bit();
    let pending = regs.exti_pr_pif;
    thread.add_fn(move || {
        pending.set_bit();
        Yielded::<(), ()>(())
    });
    thread.enable_int();
    SoftIntNotifier(regs.exti_swier_swi)
}
