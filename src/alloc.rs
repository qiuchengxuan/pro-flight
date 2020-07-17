use core::mem::size_of;
use core::sync::atomic::{AtomicUsize, Ordering};

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

pub fn allocate(size: usize, dma: bool) -> Option<&'static mut [u8]> {
    if size == 0 {
        return Some(&mut []);
    }
    let word_size = core::mem::size_of::<isize>();
    let aligned_size = ((size - 1) / word_size + 1) * word_size;
    let allocate_info = unsafe { &mut ALLOCATE_INFO };
    let total = allocate_info.no_dma.len();
    if !dma && total > 0 {
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

pub fn typed_allocate<T>(_: T, size: usize, dma: bool) -> Option<&'static mut [T]> {
    let option = allocate(core::mem::size_of::<T>() * size, dma);
    option.map(|bytes| unsafe {
        core::slice::from_raw_parts_mut(&mut bytes[0] as *mut _ as *mut T, size)
    })
}

pub fn into_static<T>(t: T, dma: bool) -> Option<&'static mut T> {
    if let Some(bytes) = allocate(size_of::<T>(), dma) {
        let static_t: &'static mut T = unsafe { &mut *(&mut bytes[0] as *mut _ as *mut T) };
        *static_t = t;
        return Some(static_t);
    }
    None
}

pub fn available() -> (usize, usize) {
    let info = unsafe { &mut ALLOCATE_INFO };
    let primary = info.primary.len() - info.primary_allocated.load(Ordering::Relaxed);
    let no_dma = info.no_dma.len() - info.no_dma_allocated.load(Ordering::Relaxed);
    (primary, no_dma)
}

mod test {
    #[test]
    #[serial]
    fn test_alloc() {
        static mut PRIMARY_BUFFER: [u8; 16] = [0u8; 16];
        static mut NO_DMA_BUFFER: [u8; 16] = [0u8; 16];
        unsafe { super::init(&mut PRIMARY_BUFFER, &mut NO_DMA_BUFFER) };

        assert_eq!(super::allocate(1, false), Some(&mut [0u8; 1][..]));

        let mut bytes: [u8; 5] = [1, 0, 0, 8, 6];
        assert_eq!(super::into_static(bytes, true), Some(&mut bytes));

        let mut bytes: [u8; 1] = [1];
        assert_eq!(super::into_static(bytes, true), Some(&mut bytes));

        let bytes: [u8; 2] = [2, 1];
        assert_eq!(super::into_static(bytes, true), None);
    }

    #[test]
    #[serial]
    fn test_typed_alloc() {
        static mut PRIMARY_BUFFER: [u8; 8] = [0u8; 8];
        super::init(unsafe { &mut PRIMARY_BUFFER }, &mut []);

        let mut option = super::typed_allocate(0u32, 2, false);
        if let Some(ref mut slice) = option {
            slice.copy_from_slice(&[0xDEADBEEF, 0xCAFEFEED]);
        }
        assert_eq!(option, Some(&mut [0xDEADBEEF, 0xCAFEFEED][..]));
    }
}
