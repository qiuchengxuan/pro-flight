pub trait PersistDatastore {
    fn load<'a, T: From<&'a [u32]>>(&'a self) -> T;
    fn save<T: AsRef<[u32]>>(&mut self, t: &T);
}
