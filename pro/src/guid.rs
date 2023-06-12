use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub struct Guid {
    pub engine: u8,
    pub type_: u16,
    pub platform: u8,
    pub region: u8,
    pub locale: u8,
    pub index: u32,
}

impl Guid {
    pub fn to_raw(&self) -> u64 {
        ((self.engine & 0xF) as u64) << 60
            | ((((self.type_ - 1) << 4).reverse_bits() & 0xFFF) as u64) << 48
            | ((self.platform & 0xF) as u64) << 44
            | ((self.region & 0x1F) as u64) << 39
            | ((self.locale & 0x1F) as u64) << 32
            | self.index as u64
    }
}

impl From<u64> for Guid {
    fn from(value: u64) -> Self {
        Self {
            engine: ((value >> 60) & 0xF) as u8,
            type_: ((((value >> 48) & 0xFFF) as u16).reverse_bits() >> 4) + 1,
            platform: ((value >> 44) & 0xF) as u8,
            region: ((value >> 39) & 0x1F) as u8,
            locale: ((value >> 32) & 0x1F) as u8,
            index: (value & 0xFFFFFFFF) as u32,
        }
    }
}

impl<'de> Deserialize<'de> for Guid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Guid::from(u64::deserialize(deserializer)?))
    }
}
