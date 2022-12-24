use drone_core::thr::ThrExec;
use drone_cortexm::thr::ThrNvic;
use hal::waker::Waker;

pub struct SoftThread<T>(T);

impl<T: ThrExec> Waker for SoftThread<T> {
    fn wakeup(&mut self) {
        self.0.wakeup()
    }
}

pub fn executor<T: ThrNvic + ThrExec>(thread: T) -> impl Waker {
    thread.enable_int();
    SoftThread(thread)
}
