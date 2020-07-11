pub trait DataWriter<T> {
    fn write(&self, t: T);
}

pub trait DataSource<T> {
    fn capacity(&self) -> usize;
    fn read(&mut self) -> Option<T>;
    fn read_last(&mut self) -> Option<T>;
    fn read_last_unchecked(&self) -> T;
}

pub mod overwriting;
pub mod singular;
pub mod u16_source;
