use alloc::boxed::Box;
use core::future::Future;
use core::sync::atomic::{AtomicBool, Ordering};
use core::{mem, slice};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Owner {
    CPU,
    DMA,
}

impl Into<bool> for Owner {
    fn into(self) -> bool {
        match self {
            Self::CPU => false,
            Self::DMA => true,
        }
    }
}

#[derive(Default)]
pub struct Meta {
    pub size: usize,
    pub owner: AtomicBool,
    pub transfer_done: Option<Box<dyn FnMut(&[u8]) + Send + 'static>>,
}

impl Meta {
    pub unsafe fn from_raw<'a>(pointer: usize) -> &'a mut Self {
        let address = pointer - mem::size_of::<Self>();
        &mut *(address as *mut Self)
    }

    pub unsafe fn get_bytes<'a>(&self) -> &'a [u8] {
        let address = self as *const _ as usize + mem::size_of::<Self>();
        slice::from_raw_parts(address as *const u8, self.size)
    }

    pub fn release(&mut self) {
        self.owner.store(Owner::CPU.into(), Ordering::Relaxed);
    }
}

#[repr(C)]
pub struct BufferDescriptor<W: Default + Copy, const N: usize> {
    meta: Meta,
    buffer: [W; N],
}

impl<W: Default + Copy, const N: usize> Default for BufferDescriptor<W, N> {
    fn default() -> Self {
        Self { meta: Meta { size: N, ..Default::default() }, buffer: [W::default(); N] }
    }
}

impl<W: Copy + Default, const N: usize> BufferDescriptor<W, N> {
    pub fn new(array: [W; N]) -> Self {
        Self { meta: Meta { size: N, ..Default::default() }, buffer: array }
    }

    pub fn set_transfer_done(&mut self, closure: impl FnMut(&[u8]) + Send + 'static) {
        self.meta.transfer_done = Some(Box::new(closure));
    }

    pub fn try_get_buffer(&mut self) -> Option<&mut [W]> {
        if self.meta.owner.load(Ordering::Relaxed) == Owner::CPU.into() {
            return Some(&mut self.buffer[..]);
        }
        None
    }

    pub fn take(&self) -> &[W] {
        self.meta.owner.store(Owner::DMA.into(), Ordering::Relaxed);
        &self.buffer[..]
    }
}

pub trait Peripheral {
    fn enable_dma(&mut self);
    fn address(&mut self) -> usize;
    fn word_size(&self) -> usize;
}

pub trait DMAFuture: Future<Output = ()> {}

/// Whenever tx or rx with buffer-descriptor, DMA does not take ownership of BD,
/// but requires BD lifetime lives no less than DMA lifetime,
/// when DMA lifetime ends, it should immediately stop and drop reference to BD
/// if any to ensure BD memory safety.
pub trait DMA: Send + 'static {
    type Future;

    fn setup_peripheral(&mut self, channel: u8, periph: &mut dyn Peripheral);
    fn tx<'a, W, BD, const N: usize>(&'a self, bd: BD, repeat: Option<usize>) -> Self::Future
    where
        W: Copy + Default,
        BD: AsRef<BufferDescriptor<W, N>> + 'a;
    fn rx<'a, W, BD, const N: usize>(&'a self, bd: BD, circle: bool) -> Self::Future
    where
        W: Copy + Default,
        BD: AsRef<BufferDescriptor<W, N>> + 'a;
    fn stop(&self);
}
