use core::mem::size_of;
use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Copy, Clone, PartialEq)]
pub enum AllocateType {
    Generic,
    DMA,
}

pub struct AllocateInfo {
    primary: &'static mut [u8],
    primary_allocated: AtomicUsize,
    no_dma: &'static mut [u8],
    no_dma_allocated: AtomicUsize,
}

static mut ALLOCATE_INFO: AllocateInfo = AllocateInfo {
    primary: &mut [],
    primary_allocated: AtomicUsize::new(0),
    no_dma: &mut [],
    no_dma_allocated: AtomicUsize::new(0),
};

pub fn init(primary: &'static mut [u8], no_dma: &'static mut [u8]) {
    unsafe {
        ALLOCATE_INFO = AllocateInfo {
            primary,
            primary_allocated: AtomicUsize::new(0),
            no_dma,
            no_dma_allocated: AtomicUsize::new(0),
        }
    };
}

pub fn allocate(size: usize, alloc_type: AllocateType) -> Option<&'static mut [u8]> {
    let word_size = core::mem::size_of::<isize>();
    let aligned_size = ((size - 1) / word_size + 1) * word_size;
    let allocate_info = unsafe { &mut ALLOCATE_INFO };
    let total = allocate_info.no_dma.len();
    if alloc_type == AllocateType::Generic && total > 0 {
        let no_dma_allocated = &mut allocate_info.no_dma_allocated;
        loop {
            let allocated = no_dma_allocated.load(Ordering::Acquire);
            if total - allocated < aligned_size {
                break;
            }
            let new = allocated + aligned_size;
            if no_dma_allocated.compare_and_swap(allocated, new, Ordering::SeqCst) == allocated {
                return Some(&mut allocate_info.no_dma[allocated..allocated + size]);
            }
        }
    }

    let total = allocate_info.primary.len();
    let primary_allocated = &mut allocate_info.primary_allocated;
    loop {
        let allocated = primary_allocated.load(Ordering::Acquire);
        if total - allocated < aligned_size {
            break;
        }
        let new = allocated + aligned_size;
        if primary_allocated.compare_and_swap(allocated, new, Ordering::SeqCst) == allocated {
            return Some(&mut allocate_info.primary[allocated..allocated + size]);
        }
    }

    None
}

pub fn allocate_zeroed(size: usize, alloc_type: AllocateType) -> Option<&'static mut [u8]> {
    allocate(size, alloc_type).map(|bytes| {
        bytes.iter_mut().for_each(|b| *b = 0);
        bytes
    })
}

pub fn into_static<T>(t: T, alloc_type: AllocateType) -> Option<&'static mut T> {
    if let Some(bytes) = allocate(size_of::<T>(), alloc_type) {
        let static_t: &'static mut T = unsafe { &mut *(&mut bytes[0] as *mut _ as *mut T) };
        *static_t = t;
        return Some(static_t);
    }
    None
}

mod test {
    #[test]
    #[serial]
    fn test_alloc() {
        use super::AllocateType;

        static mut PRIMARY_BUFFER: [u8; 16] = [0u8; 16];
        static mut NO_DMA_BUFFER: [u8; 16] = [0u8; 16];
        unsafe { super::init(&mut PRIMARY_BUFFER, &mut NO_DMA_BUFFER) };

        assert_eq!(super::allocate(1, AllocateType::Generic), Some(&mut [0u8; 1][..]));

        let mut bytes: [u8; 5] = [1, 0, 0, 8, 6];
        assert_eq!(super::into_static(bytes, AllocateType::DMA), Some(&mut bytes));

        let mut bytes: [u8; 1] = [1];
        assert_eq!(super::into_static(bytes, AllocateType::DMA), Some(&mut bytes));

        let bytes: [u8; 2] = [2, 1];
        assert_eq!(super::into_static(bytes, AllocateType::DMA), None);
    }
}
