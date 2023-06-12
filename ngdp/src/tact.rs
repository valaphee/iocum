use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

use byteorder::{BigEndian, ReadBytesExt};
use md5::{Digest, Md5};

use crate::{Error, Result};

pub struct Record {
    c_key: Vec<u8>,
    e_key: Option<Vec<u8>>,
}

impl Record {
    pub fn parse(value: &str) -> Self {
        let mut values = value.split(' ');
        Self {
            c_key: hex::decode(values.next().unwrap()).unwrap(),
            e_key: values.next().map(|e| hex::decode(e).unwrap()),
        }
    }
}

pub struct BuildConfig {
    root: Record,
    encoding: Record,
}

impl BuildConfig {
    pub fn parse<R: Read>(input: &mut R) -> Result<Self> {
        let mut entries = HashMap::new();
        for line in BufReader::new(input).lines() {
            let line = line.unwrap();
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let mut entry = line.split('=');
            entries.insert(
                entry.next().unwrap().trim().to_owned(),
                entry.next().unwrap().trim().to_owned(),
            );
        }
        Ok(Self {
            root: Record::parse(&entries["root"]),
            encoding: Record::parse(&entries["encoding"]),
        })
    }
}

pub struct Encoding {
    c_e_keys: HashMap<Vec<u8>, EncodingCEKeyEntry>,
    e_key_specs: HashMap<Vec<u8>, EncodingEKeySpecEntry>,
}

impl Encoding {
    pub fn decode<R: Read>(input: &mut R) -> Result<Self> {
        if input.read_u16::<BigEndian>()? != u16::from_be_bytes(*b"EN") {
            return Err(Error::Unsupported);
        }
        if input.read_u8()? != 1 {
            return Err(Error::Unsupported);
        }
        let c_key_size = input.read_u8()?;
        let e_key_size = input.read_u8()?;
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
            c_e_key_pages.push(EncodingPage::decode(input, c_key_size)?);
        }
        let mut c_e_keys = HashMap::new();
        for c_e_key_page in c_e_key_pages {
            let mut c_e_key_page_data = vec![0; c_e_key_page_size as usize * 0x400];
            input.read_exact(&mut c_e_key_page_data)?;
            let mut c_e_key_page_md5 = Md5::new();
            c_e_key_page_md5.update(&c_e_key_page_data);
            if c_e_key_page.md5 != c_e_key_page_md5.finalize().as_slice() {
                return Err(Error::IntegrityError);
            }
            let mut c_e_key_page_data = c_e_key_page_data.as_slice();
            while let Ok(c_to_e_key) =
                EncodingCEKeyEntry::decode(&mut c_e_key_page_data, c_key_size, e_key_size)
            {
                c_e_keys.insert(c_to_e_key.c_key.clone(), c_to_e_key);
            }
        }

        let mut e_key_spec_pages = Vec::with_capacity(e_key_spec_page_count as usize);
        for _ in 0..e_key_spec_page_count {
            e_key_spec_pages.push(EncodingPage::decode(input, e_key_size)?);
        }
        let mut e_key_specs = HashMap::new();
        for e_key_spec_page in e_key_spec_pages {
            let mut e_key_spec_page_data = vec![0; e_key_spec_page_size as usize * 0x400];
            input.read_exact(&mut e_key_spec_page_data)?;
            let mut e_key_spec_page_md5 = Md5::new();
            e_key_spec_page_md5.update(&e_key_spec_page_data);
            if e_key_spec_page.md5 != e_key_spec_page_md5.finalize().as_slice() {
                return Err(Error::IntegrityError);
            }
            let mut e_key_spec_page_data = e_key_spec_page_data.as_slice();
            while let Ok(e_key_spec) =
                EncodingEKeySpecEntry::decode(&mut e_key_spec_page_data, e_key_size, &e_specs)
            {
                e_key_specs.insert(e_key_spec.e_key.clone(), e_key_spec);
            }
        }

        Ok(Self {
            c_e_keys,
            e_key_specs,
        })
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.c_e_keys
            .get(key)
            .and_then(|c_to_e_key| c_to_e_key.e_keys.first().cloned())
    }
}

struct EncodingPage {
    first_key: Vec<u8>,
    md5: [u8; 0x10],
}

impl EncodingPage {
    fn decode<R: Read>(input: &mut R, key_size: u8) -> Result<Self> {
        Ok(Self {
            first_key: {
                let mut first_key = vec![0; key_size as usize];
                input.read_exact(&mut first_key)?;
                first_key
            },
            md5: {
                let mut md5 = [0; 0x10];
                input.read_exact(&mut md5)?;
                md5
            },
        })
    }
}

struct EncodingCEKeyEntry {
    c_key: Vec<u8>,
    c_size: u64,
    e_keys: Vec<Vec<u8>>,
}

impl EncodingCEKeyEntry {
    fn decode<R: Read>(input: &mut R, c_key_size: u8, e_key_size: u8) -> Result<Self> {
        let e_key_count = input.read_u8()?;
        let c_size = input.read_uint::<BigEndian>(5)?;
        let mut c_key = vec![0; c_key_size as usize];
        input.read_exact(&mut c_key)?;
        let mut e_keys = Vec::with_capacity(e_key_count as usize);
        for _ in 0..e_key_count {
            let mut e_key = vec![0; e_key_size as usize];
            input.read_exact(&mut e_key)?;
            e_keys.push(e_key);
        }
        Ok(Self {
            c_key,
            c_size,
            e_keys,
        })
    }
}

struct EncodingEKeySpecEntry {
    e_key: Vec<u8>,
    e_size: u64,
    e_spec: String,
}

impl EncodingEKeySpecEntry {
    fn decode<R: Read>(input: &mut R, e_key_size: u8, e_specs: &[String]) -> Result<Self> {
        Ok(Self {
            e_key: {
                let mut e_key = vec![0; e_key_size as usize];
                input.read_exact(&mut e_key)?;
                e_key
            },
            e_spec: e_specs
                .get(input.read_u32::<BigEndian>()? as usize)
                .unwrap_or(&"".to_string())
                .clone(),
            e_size: input.read_uint::<BigEndian>(5)?,
        })
    }
}

fn read_asciiz<R: Read>(input: &mut R) -> Result<String> {
    let mut data = Vec::new();
    loop {
        let value = input.read_u8()?;
        if value == 0 {
            break;
        }
        data.push(value as char);
    }
    Ok(data.iter().collect())
}
