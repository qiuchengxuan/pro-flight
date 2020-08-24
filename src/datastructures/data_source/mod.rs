use core::marker::PhantomData;

pub trait DataWriter<T> {
    fn write(&self, t: T);
}

pub trait WithCapacity {
    fn capacity(&self) -> usize;
}

pub trait OptionData<T> {
    fn read(&mut self) -> Option<T>;
}

pub trait StaticData<T> {
    fn read(&mut self) -> T;
}

pub trait AgingStaticData<T> {
    fn read(&mut self, max_age: usize) -> Option<T>;
}

pub struct NoDataSource<T> {
    t: PhantomData<T>,
}

impl<T> NoDataSource<T> {
    pub fn new() -> Self {
        Self { t: PhantomData }
    }
}

impl<T> WithCapacity for NoDataSource<T> {
    fn capacity(&self) -> usize {
        0
    }
}

impl<T> OptionData<T> for NoDataSource<T> {
    fn read(&mut self) -> Option<T> {
        None
    }
}

impl<T: Default> StaticData<T> for NoDataSource<T> {
    fn read(&mut self) -> T {
        T::default()
    }
}

impl<T: Default> AgingStaticData<T> for NoDataSource<T> {
    fn read(&mut self, _: usize) -> Option<T> {
        None
    }
}

pub mod overwriting;
pub mod singular;
pub mod u16_source;
