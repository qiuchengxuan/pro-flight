use core::mem::size_of;

pub const UBX_HEADER0: u8 = 0xB5;
pub const UBX_HEADER1: u8 = 0x62;
pub const CHECKSUM_SIZE: usize = 2;
pub const PAYLOAD_OFFSET: usize = size_of::<Message<()>>() - CHECKSUM_SIZE;

pub enum PayloadType {
    NavPosPvt,
}

impl PayloadType {
    pub fn try_from(class: u8, id: u8) -> Option<Self> {
        match (class, id) {
            (0x1, 0x7) => Some(Self::NavPosPvt),
            _ => None,
        }
    }
}

#[repr(C)]
pub struct Message<T> {
    pub class: u8,
    pub id: u8,
    pub length: u16,
    pub payload: T,
    pub checksum_a: u8,
    pub checksum_b: u8,
}

impl<T> Message<T> {
    pub fn payload_type(&self) -> Option<PayloadType> {
        PayloadType::try_from(self.class, self.id)
    }

    pub fn length(&self) -> usize {
        u16::from_le(self.length) as usize
    }

    pub fn validate_checksum(&self) -> bool {
        let mut a: u8 = 0;
        let mut b: u8 = 0;
        let size = 4 + size_of::<T>();
        let bytes: &[u8] = unsafe { core::slice::from_raw_parts(&self.class, size) };
        for &byte in bytes.iter() {
            a = a.wrapping_add(byte);
            b = b.wrapping_add(a);
        }
        a == self.checksum_a && b == self.checksum_b
    }
}
