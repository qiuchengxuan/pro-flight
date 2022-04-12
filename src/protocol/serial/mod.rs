use alloc::boxed::Box;

use crate::config::peripherals::serial::Config;

pub trait Receiver: Send {
    fn chunk_size(&self) -> usize;
    fn receive(&mut self, bytes: &[u8]);
    fn reset(&mut self);
}

pub mod gnss;
pub mod rc;

pub fn make_receiver(config: &Config) -> Option<Box<dyn Receiver>> {
    match config {
        Config::RC(rc) => Some(Box::new(rc::RemoteControl::from(rc))),
        Config::GNSS(gnss) => Some(Box::new(gnss::GNSSReceiver::from(gnss.protocol))),
    }
}
