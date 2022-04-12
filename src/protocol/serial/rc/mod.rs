pub mod sbus;

use crate::{
    config::peripherals::serial::RemoteControl as Config,
    protocol::{rc::ControlMatrix, serial::Receiver},
};

enum Protocol {
    SBUS(sbus::SBUS),
}

pub struct RemoteControl {
    protocol: Protocol,
    matrix: ControlMatrix,
}

impl Receiver for RemoteControl {
    fn chunk_size(&self) -> usize {
        match self.protocol {
            Protocol::SBUS(_) => sbus::CHUNK_SIZE,
        }
    }

    fn receive(&mut self, bytes: &[u8]) {
        let raw = match &mut self.protocol {
            Protocol::SBUS(ref mut sbus) => sbus.receive(bytes),
        };
        if let Some(raw) = raw {
            self.matrix.read(&raw.channels);
        }
    }

    fn reset(&mut self) {
        match &mut self.protocol {
            Protocol::SBUS(ref mut sbus) => sbus.reset(),
        }
    }
}

impl From<&Config> for RemoteControl {
    fn from(config: &Config) -> Self {
        let protocol = match config {
            Config::SBUS(sbus) => Protocol::SBUS(sbus::SBUS::new(sbus.fast)),
        };
        Self { protocol, matrix: ControlMatrix::default() }
    }
}
