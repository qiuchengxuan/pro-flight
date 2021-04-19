use core::time::Duration;

use embedded_hal::timer::CountDown;

pub trait Notifier {
    fn notify(&mut self);
}

pub struct TimedNotifier<C, N> {
    count_down: C,
    notifier: N,
    interval: Duration,
}

impl<T: From<Duration>, C: CountDown<Time = T>, N: Notifier> TimedNotifier<C, N> {
    pub fn new(notifier: N, count_down: C, interval: Duration) -> Self {
        Self { count_down, notifier, interval }
    }

    pub fn notify_now(&mut self) {
        self.notifier.notify()
    }
}

impl<T: From<Duration>, C: CountDown<Time = T>, N: Notifier> Notifier for TimedNotifier<C, N> {
    fn notify(&mut self) {
        if !self.count_down.wait().is_ok() {
            return;
        }
        self.count_down.start(self.interval);
        self.notifier.notify()
    }
}
