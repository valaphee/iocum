use std::{collections::HashMap, io::Read};

use byteorder::{BigEndian, ReadBytesExt};

use crate::{md5, read_asciiz, Error, Result};

pub struct Encoding {
    c_e_keys: HashMap<u128, EncodingCEKeyEntry>,
    e_key_specs: HashMap<u128, EncodingEKeySpecEntry>,
}

impl Encoding {
    pub fn decode<R: Read>(input: &mut R) -> Result<Self> {
        if input.read_u16::<BigEndian>()? != u16::from_be_bytes(*b"EN") {
            return Err(Error::Unsupported);
        }
        let version = input.read_u8()?;
        if version != 1 {
            return Err(Error::Unsupported);
        }
        let c_key_size = input.read_u8()?;
        if c_key_size != 16 {
            return Err(Error::Unsupported);
        }
        let e_key_size = input.read_u8()?;
        if e_key_size != 16 {
            return Err(Error::Unsupported);
        }
        let c_e_key_page_size = input.read_u16::<BigEndian>()?;
        let e_key_spec_page_size = input.read_u16::<BigEndian>()?;
        let c_e_key_page_count = input.read_u32::<BigEndian>()?;
        let e_key_spec_page_count = input.read_u32::<BigEndian>()?;
        if input.read_u8()? != 0 {
            return Err(Error::Unsupported);
        }

        let mut e_spec_block = vec![0; input.read_u32::<BigEndian>()? as usize];
        input.read_exact(&mut e_spec_block)?;
        let mut e_specs_data = e_spec_block.as_slice();
        let mut e_specs = Vec::new();
        while let Ok(e_spec) = read_asciiz(&mut e_specs_data) {
            e_specs.push(e_spec);
        }

        let mut c_e_key_pages = Vec::with_capacity(c_e_key_page_count as usize);
        for _ in 0..c_e_key_page_count {
            c_e_key_pages.push(EncodingPage::decode(input)?);
        }
        let mut c_e_keys = HashMap::new();
        for c_e_key_page in c_e_key_pages {
            let mut c_e_key_page_data = vec![0; c_e_key_page_size as usize * 1024];
            input.read_exact(&mut c_e_key_page_data)?;
            let mut c_e_key_page_data = c_e_key_page_data.as_slice();
            if c_e_key_page.md5 != md5(c_e_key_page_data) {
                return Err(Error::IntegrityError);
            }
            while let Ok(c_to_e_key) = EncodingCEKeyEntry::decode(&mut c_e_key_page_data) {
                c_e_keys.insert(c_to_e_key.c_key, c_to_e_key);
            }
        }

        let mut e_key_spec_pages = Vec::with_capacity(e_key_spec_page_count as usize);
        for _ in 0..e_key_spec_page_count {
            e_key_spec_pages.push(EncodingPage::decode(input)?);
        }
        let mut e_key_specs = HashMap::new();
        for e_key_spec_page in e_key_spec_pages {
            let mut e_key_spec_page_data = vec![0; e_key_spec_page_size as usize * 1024];
            input.read_exact(&mut e_key_spec_page_data)?;
            let mut e_key_spec_page_data = e_key_spec_page_data.as_slice();
            if e_key_spec_page.md5 != md5(e_key_spec_page_data) {
                return Err(Error::IntegrityError);
            }
            while let Ok(e_key_spec) =
                EncodingEKeySpecEntry::decode(&mut e_key_spec_page_data, &e_specs)
            {
                e_key_specs.insert(e_key_spec.e_key, e_key_spec);
            }
        }

        Ok(Self {
            c_e_keys,
            e_key_specs,
        })
    }

    pub fn get(&self, key: u128) -> Option<u128> {
        self.c_e_keys
            .get(&key)
            .and_then(|c_to_e_key| c_to_e_key.e_keys.first().cloned())
    }
}

struct EncodingPage {
    first_key: u128,
    md5: u128,
}

impl EncodingPage {
    fn decode<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            first_key: input.read_u128::<BigEndian>()?,
            md5: input.read_u128::<BigEndian>()?,
        })
    }
}

struct EncodingCEKeyEntry {
    c_key: u128,
    c_size: u64,
    e_keys: Vec<u128>,
}

impl EncodingCEKeyEntry {
    fn decode<R: Read>(input: &mut R) -> Result<Self> {
        let e_key_count = input.read_u8()?;
        let c_size = input.read_uint::<BigEndian>(5)?;
        let c_key = input.read_u128::<BigEndian>()?;
        let mut e_keys = Vec::with_capacity(e_key_count as usize);
        for _ in 0..e_key_count {
            e_keys.push(input.read_u128::<BigEndian>()?);
        }
        Ok(Self {
            c_key,
            c_size,
            e_keys,
        })
    }
}

struct EncodingEKeySpecEntry {
    e_key: u128,
    e_size: u64,
    e_spec: String,
}

impl EncodingEKeySpecEntry {
    fn decode<R: Read>(input: &mut R, e_specs: &[String]) -> Result<Self> {
        Ok(Self {
            e_key: input.read_u128::<BigEndian>()?,
            e_spec: e_specs
                .get(input.read_u32::<BigEndian>()? as usize)
                .unwrap_or(&"".to_string())
                .clone(),
            e_size: input.read_uint::<BigEndian>(5)?,
        })
    }
}
