use drone_core::thr::ThrExec;
use drone_cortexm::thr::ThrNvic;
use hal::thread::Thread;

pub struct SoftThread<T>(T);

impl<T: ThrExec> Thread for SoftThread<T> {
    fn wakeup(&mut self) {
        self.0.wakeup()
    }
}

pub fn into_thread<T: ThrNvic + ThrExec>(thread: T) -> impl Thread {
    thread.enable_int();
    SoftThread(thread)
}
