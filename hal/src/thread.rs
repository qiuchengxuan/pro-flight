use core::time::Duration;

use embedded_hal::timer::CountDown;

pub trait Thread {
    fn wakeup(&mut self);
}

pub struct Schedule<C, T> {
    count_down: C,
    thread: T,
    interval: Duration,
}

impl<D: From<Duration>, C: CountDown<Time = D>, T: Thread> Schedule<C, T> {
    pub fn new(thread: T, count_down: C, interval: Duration) -> Self {
        Self { count_down, thread, interval }
    }

    pub fn wakeup_now(&mut self) {
        self.thread.wakeup()
    }
}

impl<D: From<Duration>, C: CountDown<Time = D>, T: Thread> Thread for Schedule<C, T> {
    fn wakeup(&mut self) {
        if !self.count_down.wait().is_ok() {
            return;
        }
        self.count_down.start(self.interval);
        self.thread.wakeup()
    }
}
