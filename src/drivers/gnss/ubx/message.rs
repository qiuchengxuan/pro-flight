pub const UBX_HEADER: [u8; 2] = [0xB5, 0x62];

pub trait ClassAndID {
    fn class_and_id() -> (u8, u8);
}

#[repr(C)]
pub struct Message<T> {
    _padding: u16,
    pub header: u16,
    pub class: u8,
    pub id: u8,
    pub length: u16,
    pub payload: T,
    pub checksum: u16,
}

impl<T: ClassAndID> Message<T> {
    pub fn valid_class_and_id(&self) -> bool {
        let (class, id) = T::class_and_id();
        self.class == class && self.id == id
    }
}

impl<T> Message<T> {
    pub fn calc_checksum(&self) -> u16 {
        let mut a: u8 = 0;
        let mut b: u8 = 0;
        let size = 4 + core::mem::size_of::<T>();
        let bytes: &[u8] = unsafe { core::slice::from_raw_parts(&self.class, size) };
        for &byte in bytes.iter() {
            a = a.wrapping_add(byte);
            b = b.wrapping_add(a);
        }
        u16::to_be((a as u16) << 8 | b as u16)
    }
}
