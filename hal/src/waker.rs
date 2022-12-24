use embedded_hal::timer::CountDown;

pub trait Waker {
    fn wakeup(&mut self);
}

pub struct Schedule<D, C, T> {
    count_down: C,
    thread: T,
    interval: D,
}

impl<D, C: CountDown<Time = D>, T: Waker> Schedule<D, C, T> {
    pub fn new(thread: T, count_down: C, interval: D) -> Self {
        Self { count_down, thread, interval }
    }

    pub fn wakeup_now(&mut self) {
        self.thread.wakeup()
    }
}

impl<D: Copy, C: CountDown<Time = D>, T: Waker> Waker for Schedule<D, C, T> {
    fn wakeup(&mut self) {
        if !self.count_down.wait().is_ok() {
            return;
        }
        self.count_down.start(self.interval);
        self.thread.wakeup()
    }
}
