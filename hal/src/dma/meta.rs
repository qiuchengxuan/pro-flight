use core::{
    mem, slice,
    sync::atomic::{AtomicU8, Ordering},
};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Owner {
    Free,
    CPU,
    DMA,
}

impl From<u8> for Owner {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Free,
            1 => Self::CPU,
            2 => Self::DMA,
            _ => unreachable!(),
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

#[repr(C)]
#[derive(Default)]
pub struct Meta<W: 'static> {
    pub callback: Option<&'static mut dyn FnMut(TransferResult<W>)>,
    pub owner: AtomicU8,
    pub size: usize,
}

unsafe impl<W> Sync for Meta<W> {}

impl<W> Meta<W> {
    pub unsafe fn from_raw<'a>(pointer: usize) -> &'a mut Self {
        let address = pointer - mem::size_of::<Self>();
        &mut *(address as *mut Self)
    }

    pub unsafe fn get_buffer<'a>(&self) -> &'a [W] {
        let address = self as *const _ as usize + mem::size_of::<Self>();
        slice::from_raw_parts(address as *const W, self.size)
    }

    pub unsafe fn release(&self) {
        self.owner.store(Owner::Free as u8, Ordering::Relaxed);
    }

    pub fn take_ownership(&self, owner: Owner) -> Result<(), Owner> {
        let ord = Ordering::Relaxed;
        self.owner
            .compare_exchange(Owner::Free as u8, owner as u8, ord, ord)
            .map(|_| ())
            .map_err(|v| Owner::from(v))
    }
}
