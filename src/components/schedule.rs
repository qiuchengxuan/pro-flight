use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ptr::{read_volatile, write_volatile};

pub type Rate = usize;

pub trait Schedulable {
    fn schedule(&mut self) -> bool;
    fn rate(&self) -> Rate;
}

pub struct TaskInfo {
    counter: usize,
    interval: usize,
}

pub struct Scheduler {
    schedulables: Vec<Box<dyn Schedulable>>,
    rate: Rate,
    task_infos: Vec<TaskInfo>,
    running: bool,
}

impl Scheduler {
    pub fn new(schedulables: Vec<Box<dyn Schedulable>>, rate: Rate) -> Self {
        let mut task_infos: Vec<TaskInfo> = Vec::with_capacity(schedulables.len());
        for schedulable in schedulables.iter() {
            let interval = rate / schedulable.rate();
            task_infos.push(TaskInfo { counter: 0, interval });
        }
        Self { schedulables, rate, task_infos, running: false }
    }
}

impl Schedulable for Scheduler {
    fn schedule(&mut self) -> bool {
        for i in 0..self.schedulables.len() {
            let task_info = &mut self.task_infos[i];
            task_info.counter += 1;
        }

        if unsafe { read_volatile(&self.running) } {
            // in case of re-enter
            return true;
        }

        unsafe { write_volatile(&mut self.running, true) };
        for i in 0..self.schedulables.len() {
            let task_info = &mut self.task_infos[i];
            if task_info.counter < task_info.interval {
                continue;
            }
            if self.schedulables[i].schedule() {
                task_info.counter = 0;
            }
        }
        unsafe { write_volatile(&mut self.running, false) };
        true
    }

    fn rate(&self) -> Rate {
        self.rate
    }
}
