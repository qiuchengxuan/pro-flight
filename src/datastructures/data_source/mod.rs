use core::marker::PhantomData;

pub trait DataWriter<T> {
    fn write(&self, t: T);
}

pub trait DataSource<T> {
    fn capacity(&self) -> usize;
    fn read(&mut self) -> Option<T>;
    fn read_last(&mut self) -> Option<T>;
    fn read_last_unchecked(&self) -> T;
}

pub struct NoDataSource<T> {
    t: PhantomData<T>,
}

impl<T> NoDataSource<T> {
    pub fn new() -> Self {
        Self { t: PhantomData }
    }
}

impl<T: Default> DataSource<T> for NoDataSource<T> {
    fn capacity(&self) -> usize {
        0
    }

    fn read(&mut self) -> Option<T> {
        None
    }

    fn read_last(&mut self) -> Option<T> {
        None
    }

    fn read_last_unchecked(&self) -> T {
        T::default()
    }
}

pub mod overwriting;
pub mod singular;
pub mod u16_source;
