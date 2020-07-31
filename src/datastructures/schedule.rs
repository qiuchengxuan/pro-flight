pub type Hertz = usize;

pub trait Schedulable {
    fn schedule(&mut self);
    fn rate(&self) -> Hertz;
}

pub trait Schedulables {
    fn foreach(&mut self, f: impl FnMut(&mut dyn Schedulable));
    fn len() -> usize;
}

macro_rules! schedulables {
    () => ();
    ($idx1:tt $(,$idx:tt)+ -> $S0:ident $(, $S:ident)+) => {
        impl<$S0: Schedulable $(,$S: Schedulable)+> Schedulables for ($S0 $(,$S)+) {
            fn foreach(&mut self, mut f: impl FnMut(&mut dyn Schedulable)) {
                f(&mut self.0);
                $(f(&mut self.$idx);)+
            }

            fn len() -> usize {
                1 $( + $idx / $idx)+
            }
        }
    }
}

schedulables! {0, 1, 2, 3, 4, 5 -> S0, S1, S2, S3, S4, S5}

pub struct Scheduler<S> {
    schedulables: S,
    rate: Hertz,
    counter: usize,
}

impl<S: Schedulables> Scheduler<S> {
    pub fn new(schedulables: S, rate: Hertz) -> Self {
        Self { schedulables, rate, counter: 0 }
    }
}

impl<S: Schedulables> Schedulable for Scheduler<S> {
    fn schedule(&mut self) {
        let scheduler_rate = self.rate;
        let counter = self.counter;
        self.schedulables.foreach(|s| {
            let schedule = match s.rate() {
                0 => false,
                1 => counter == 0,
                _ => s.rate() == scheduler_rate || (counter % (scheduler_rate / s.rate()) == 0),
            };
            if schedule {
                s.schedule();
            }
        });
        self.counter += 1;
        if self.counter >= self.rate {
            self.counter = 0
        }
    }

    fn rate(&self) -> Hertz {
        self.rate
    }
}
