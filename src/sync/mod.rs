use core::marker::PhantomData;

pub trait DataWriter<T> {
    fn write(&self, t: T);
}

pub trait WithCapacity {
    fn capacity(&self) -> usize;
}

pub trait DataReader<T> {
    fn get(&mut self) -> Option<T>;
    fn get_last(&mut self) -> Option<T>;
}

pub trait AgingDataReader<T> {
    fn get_aging_last(&mut self, max_age: usize) -> Option<T>;
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

impl<T> DataReader<T> for NoDataSource<T> {
    fn get(&mut self) -> Option<T> {
        None
    }

    fn get_last(&mut self) -> Option<T> {
        None
    }
}

impl<T: Default> AgingDataReader<T> for NoDataSource<T> {
    fn get_aging_last(&mut self, _: usize) -> Option<T> {
        None
    }
}

pub mod overwriting;
pub mod singular;
