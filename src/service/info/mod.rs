pub trait Writer<T> {
    fn write(&self, t: T);
}

pub trait WithCapacity {
    fn capacity(&self) -> usize;
}

pub trait Reader<T> {
    fn get(&mut self) -> Option<T>;
    fn get_last(&mut self) -> Option<T>;
}

pub trait AgingReader<T> {
    fn get_aging_last(&mut self, max_age: usize) -> Option<T>;
}

pub mod bulletin;
pub mod stream;
