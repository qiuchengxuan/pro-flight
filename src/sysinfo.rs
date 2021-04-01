#[derive(Copy, Clone, PartialEq)]
pub enum RebootReason {
    Normal,
    Bootloader,
}

impl Default for RebootReason {
    fn default() -> Self {
        Self::Normal
    }
}

#[repr(C, align(4))]
#[derive(Default)]
pub struct SystemInfo {
    pub reboot_reason: RebootReason,
}

impl AsRef<[u32]> for SystemInfo {
    fn as_ref(&self) -> &[u32] {
        let size = core::mem::size_of::<Self>() / 4;
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u32, size) }
    }
}

impl From<&[u32]> for SystemInfo {
    fn from(slice: &[u32]) -> Self {
        let mut info = Self::default();
        let size = core::mem::size_of::<Self>() / 4;
        let v = unsafe { core::slice::from_raw_parts_mut(&mut info as *mut _ as *mut u32, size) };
        v.copy_from_slice(&slice[..size]);
        info
    }
}
