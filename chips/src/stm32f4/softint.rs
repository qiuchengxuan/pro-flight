use core::task::{Context, RawWaker, RawWakerVTable, Waker};

use drone_core::fib::{new_fn, Yielded};
use drone_cortexm::reg::field::WRwRegFieldBitAtomic;
use drone_cortexm::reg::prelude::*;
use drone_cortexm::thr::ThrNvic;
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

struct DummyWaker; // We don't really need a waker

fn notifier_waker_wake(_w: &DummyWaker) {}

fn notifier_waker_clone(w: &DummyWaker) -> RawWaker {
    RawWaker::new(w as *const _ as *const (), &VTABLE)
}

const VTABLE: RawWakerVTable = unsafe {
    RawWakerVTable::new(
        |w| notifier_waker_clone(&*(w as *const DummyWaker)),
        |w| notifier_waker_wake(&*(w as *const DummyWaker)),
        |w| notifier_waker_wake(*(w as *const &DummyWaker)),
        |_w| {},
    )
};

fn make_waker(waker: *const DummyWaker) -> Waker {
    unsafe { Waker::from_raw(RawWaker::new(waker as *const (), &VTABLE)) }
}

pub fn make_soft_int<T, M, F>(thread: T, regs: ExtiPeriph<M>, mut f: F) -> impl Notifier
where
    T: ThrNvic,
    M: ExtiMap + ExtiPrPif + ExtiSwierSwiOpt + ExtiSwierSwi,
    F: FnMut(&mut Context) + Send + 'static,
{
    regs.exti_imr_im.set_bit();
    let pending = regs.exti_pr_pif;
    thread.add_fib(new_fn(move || {
        pending.set_bit();
        let waker = make_waker(&DummyWaker as *const DummyWaker);
        let mut context = Context::from_waker(&waker);
        f(&mut context);
        Yielded::<(), ()>(())
    }));
    thread.enable_int();
    SoftIntNotifier(regs.exti_swier_swi)
}
