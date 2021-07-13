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

pub enum TransferResult<'a, W> {
    Complete(&'a [W]),
    Half(&'a [W]),
}

impl<'a, W> TransferResult<'a, W> {
    pub fn half(self) -> &'a [W] {
        match self {
            Self::Half(data) => &data[..data.len() / 2],
            Self::Complete(data) => &data[data.len() / 2..],
        }
    }
}

impl<'a, W> Into<&'a [W]> for TransferResult<'a, W> {
    fn into(self) -> &'a [W] {
        match self {
            Self::Half(data) => data,
            Self::Complete(data) => data,
        }
    }
}

type Handler<W> = dyn FnMut(TransferResult<W>) + Send + 'static;

#[repr(C)]
#[derive(Default)]
pub struct Meta<W> {
    handler: Option<Box<Handler<W>>>,
    pub owner: AtomicBool,
    pub size: usize,
}

impl<W> Meta<W> {
    pub unsafe fn from_raw<'a>(pointer: usize) -> &'a mut Self {
        let address = pointer - mem::size_of::<Self>();
        &mut *(address as *mut Self)
    }

    pub unsafe fn get_buffer<'a>(&self) -> &'a [W] {
        let address = self as *const _ as usize + mem::size_of::<Self>();
        slice::from_raw_parts(address as *const W, self.size)
    }

    // unsafe because Sync
    pub unsafe fn get_handler(&mut self) -> Option<&mut Handler<W>> {
        self.handler.as_mut().map(|f| f.as_mut())
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

    pub fn set_handler(&mut self, handler: impl FnMut(TransferResult<W>) + Send + 'static) -> bool {
        if self.meta.owner.load(Ordering::Relaxed) == Owner::CPU.into() {
            self.meta.handler = Some(Box::new(handler));
            return true;
        }
        false
    }

    pub fn try_get_buffer(&mut self) -> Option<&mut [W]> {
        if self.meta.owner.load(Ordering::Relaxed) == Owner::CPU.into() {
            return Some(&mut self.buffer[..]);
        }
        None
    }

    pub fn set_size(&mut self, size: usize) {
        self.meta.size = size
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

type BD<W, const N: usize> = BufferDescriptor<W, N>;

/// Whenever tx or rx with buffer-descriptor, DMA does not take ownership of BD,
/// but requires BD lifetime lives no less than DMA lifetime,
/// when DMA lifetime ends, it should immediately stop and drop reference to BD
/// if any to ensure BD memory safety.
pub trait DMA: Send + Sync + 'static {
    type Future;

    fn setup_peripheral(&mut self, channel: Channel, periph: &mut dyn Peripheral);
    fn is_busy(&self) -> bool;
    fn tx<'a, W, const N: usize>(
        &'a self,
        bd: &'a BD<W, N>,
        option: TransferOption,
    ) -> Self::Future
    where
        W: Copy + Default;
    fn rx<'a, W, const N: usize>(
        &'a self,
        bd: &'a mut BD<W, N>,
        option: TransferOption,
    ) -> Self::Future
    where
        W: Copy + Default;
    fn setup_rx<W, const N: usize>(self, bd: &'static mut BD<W, N>, option: TransferOption)
    where
        W: Copy + Default;
    fn stop(&self);
}
