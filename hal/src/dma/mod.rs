pub mod bd;
pub mod meta;

use core::future::Future;

pub use bd::BufferDescriptor;
pub use meta::{Meta, TransferResult};

pub trait Peripheral {
    fn enable_dma(&mut self);
    fn address(&mut self) -> usize;
    fn word_size(&self) -> usize;
}

pub trait DMAFuture: Future<Output = ()> {}

#[derive(Copy, Clone, Default)]
pub struct TransferOption {
    /// If not specified, default to buffer-descriptor size
    pub size: Option<usize>,
    /// Memory address won't increase
    pub fixed: bool,
    /// Restart another transfer when transfer completes
    pub circle: bool,
    /// Immediatly retrieve data when half buffer filled
    pub enable_half: bool,
}

impl TransferOption {
    pub fn repeat() -> Self {
        Self { fixed: true, ..Default::default() }
    }

    pub fn circle() -> Self {
        Self { circle: true, ..Default::default() }
    }

    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn enable_half(mut self) -> Self {
        self.enable_half = true;
        self
    }
}

pub type Channel = u8;

pub type BD<W, const N: usize> = BufferDescriptor<W, N>;

/// Whenever tx or rx with buffer-descriptor, DMA does not take ownership of BD,
/// but requires BD lifetime lives no less than DMA lifetime,
/// when DMA lifetime ends, it should immediately stop and drop reference to BD
/// if any to ensure BD memory safety.
pub trait DMA: Send + Sync + 'static {
    type Future: DMAFuture;

    fn setup_peripheral(&mut self, channel: Channel, periph: &mut dyn Peripheral);
    fn is_busy(&self) -> bool;
    fn tx<'a, W, const N: usize>(&'a self, bd: &'a BD<W, N>, o: TransferOption) -> Self::Future
    where
        W: Copy + Default + 'static;
    fn rx<'a, W, const N: usize>(&'a self, bd: &'a mut BD<W, N>, o: TransferOption) -> Self::Future
    where
        W: Copy + Default + 'static;
    fn setup_rx<W, const N: usize>(self, bd: &'static mut BD<W, N>, option: TransferOption)
    where
        W: Copy + Default + 'static;
    fn stop(&self);
}
