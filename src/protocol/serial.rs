pub trait Receiver: Send {
    fn receive(&mut self, bytes: &[u8]);
}
