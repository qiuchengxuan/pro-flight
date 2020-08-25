use alloc::vec::Vec;

pub type Rate = usize;

pub trait Schedulable {
    fn schedule(&mut self) -> bool;
    fn rate(&self) -> Rate;
}

pub trait Schedulables {
    fn get(&mut self, index: usize) -> Option<&mut dyn Schedulable>;
    fn len() -> usize;
}

macro_rules! schedulables {
    () => ();
    ($idx1:tt $(,$idx:tt)+ -> $S0:ident $(, $S:ident)+) => {
        impl<$S0: Schedulable $(,$S: Schedulable)+> Schedulables for ($S0 $(,$S)+) {
            fn get(&mut self, index: usize) -> Option<&mut dyn Schedulable> {
                match index {
                    0 => Some(&mut self.0),
                    $($idx => Some(&mut self.$idx),)+
                    _ => None
                }
            }

            fn len() -> usize {
                1 $( + $idx / $idx)+
            }
        }
    }
}

schedulables! {0, 1, 2, 3, 4, 5, 6 -> S0, S1, S2, S3, S4, S5, S6}

pub struct TaskInfo {
    counter: usize,
    interval: usize,
}

pub struct Scheduler<S> {
    schedulables: S,
    rate: Rate,
    task_infos: Vec<TaskInfo>,
    running: bool,
}

impl<S: Schedulables> Scheduler<S> {
    pub fn new(mut schedulables: S, rate: Rate) -> Self {
        let mut task_infos: Vec<TaskInfo> = Vec::with_capacity(S::len());
        for i in 0..S::len() {
            let interval = rate / schedulables.get(i).unwrap().rate();
            task_infos.push(TaskInfo { counter: 0, interval });
        }
        Self { schedulables, rate, task_infos, running: false }
    }
}

impl<S: Schedulables> Schedulable for Scheduler<S> {
    fn schedule(&mut self) -> bool {
        for i in 0..S::len() {
            let task_info = &mut self.task_infos[i];
            task_info.counter += 1;
        }

        if self.running {
            // in case of re-enter
            return true;
        }

        self.running = true;
        for i in 0..S::len() {
            let task_info = &mut self.task_infos[i];
            if task_info.counter < task_info.interval {
                continue;
            }
            if self.schedulables.get(i).unwrap().schedule() {
                task_info.counter = 0;
            }
        }
        self.running = false;
        true
    }

    fn rate(&self) -> Rate {
        self.rate
    }
}
