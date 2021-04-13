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

#[repr(C)]
#[derive(Default)]
pub struct Meta<W> {
    transfer_done: Option<Box<dyn FnMut(&[W]) + Send + 'static>>,
    pub owner: AtomicBool,
    pub size: usize,
}

impl<W> Meta<W> {
    pub unsafe fn from_raw<'a>(pointer: usize) -> &'a mut Self {
        let address = pointer - mem::size_of::<Self>();
        &mut *(address as *mut Self)
    }

    pub unsafe fn get_data<'a>(&self) -> &'a [W] {
        let address = self as *const _ as usize + mem::size_of::<Self>();
        slice::from_raw_parts(address as *const W, self.size)
    }

    // unsafe because Sync
    pub unsafe fn get_transfer_done(&mut self) -> Option<&mut (dyn FnMut(&[W]) + Send + 'static)> {
        self.transfer_done.as_mut().map(|f| f.as_mut())
    }

    pub fn release(&mut self) {
        self.owner.store(Owner::CPU.into(), Ordering::Relaxed);
    }
}

unsafe impl<W> Sync for Meta<W> {}

#[repr(C)]
pub struct BufferDescriptor<W: Default + Copy, const N: usize> {
    meta: Meta<W>,
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

    pub fn set_transfer_done(&mut self, closure: impl FnMut(&[W]) + Send + 'static) {
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

#[derive(Copy, Clone, Default)]
pub struct TransferOption {
    /// if not specified, default to buffer-descriptor size
    pub size: Option<usize>,
    /// memory address won't increase
    pub fixed: bool,
    /// restart another transfer when transfer completes
    pub circle: bool,
}

impl TransferOption {
    pub fn sized(size: usize) -> Self {
        Self { size: Some(size), ..Default::default() }
    }

    pub fn repeat(size: usize) -> Self {
        Self { size: Some(size), fixed: true, ..Default::default() }
    }

    pub fn circle() -> Self {
        Self { circle: true, ..Default::default() }
    }
}

/// Whenever tx or rx with buffer-descriptor, DMA does not take ownership of BD,
/// but requires BD lifetime lives no less than DMA lifetime,
/// when DMA lifetime ends, it should immediately stop and drop reference to BD
/// if any to ensure BD memory safety.
pub trait DMA: Send + Sync + 'static {
    type Future;

    fn setup_peripheral(&mut self, channel: u8, periph: &mut dyn Peripheral);
    fn is_busy(&self) -> bool;
    fn tx<'a, W, BD, const N: usize>(&'a self, bd: BD, option: TransferOption) -> Self::Future
    where
        W: Copy + Default,
        BD: AsRef<BufferDescriptor<W, N>> + 'a;
    fn rx<'a, W, BD, const N: usize>(&'a self, bd: BD, option: TransferOption) -> Self::Future
    where
        W: Copy + Default,
        BD: AsRef<BufferDescriptor<W, N>> + 'a;
    fn stop(&self);
}
