use std::{collections::HashMap, io::Read};

use byteorder::{BigEndian, ReadBytesExt};
use md5::{Digest, Md5};
use serde::Deserialize;

use crate::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct BuildInfo {
    #[serde(rename = "Branch!STRING:0")]
    pub branch: String,
    #[serde(rename = "Build Key!HEX:16", with = "hex")]
    pub build_key: [u8; 16],
    #[serde(rename = "CDN Key!HEX:16", with = "hex")]
    pub cdn_key: [u8; 16],
    #[serde(rename = "Install Key!HEX:16", with = "hex")]
    pub install_key: [u8; 16],
    #[serde(rename = "CDN Path!STRING:0")]
    pub cdn_path: String,
    #[serde(rename = "CDN Hosts!STRING:0")]
    pub cdn_hosts: String,
    #[serde(rename = "CDN Servers!STRING:0")]
    pub cdn_servers: String,
    #[serde(rename = "Tags!STRING:0")]
    pub tags: String,
    #[serde(rename = "Version!STRING:0")]
    pub version: String,
    #[serde(rename = "KeyRing!HEX:16", with = "hex")]
    pub keyring: [u8; 16],
    #[serde(rename = "Product!STRING:0")]
    pub product: String,
}

#[derive(Debug, Deserialize)]
pub struct RootFile {
    #[serde(rename = "#FILEID")]
    pub file_id: String,
    #[serde(rename = "MD5", with = "hex")]
    pub md5: [u8; 16],
    #[serde(rename = "CHUNK_ID")]
    pub chunk_id: u8,
    #[serde(rename = "PRIORITY")]
    pub priority: u8,
    #[serde(rename = "MPRIORITY")]
    pub mpriority: u8,
    #[serde(rename = "FILENAME")]
    pub file_name: String,
    #[serde(rename = "INSTALLPATH")]
    pub install_path: String,
}

#[derive(Debug)]
pub struct Encoding {
    c_to_e_keys: HashMap<Vec<u8>, EncodingCToEKey>,
    e_key_specs: HashMap<Vec<u8>, EncodingEKeySpec>,
}

impl Encoding {
    pub fn decode<R: Read>(input: &mut R) -> Result<Self> {
        if input.read_u16::<BigEndian>()? != u16::from_be_bytes(*b"EN") {
            return Err(Error::Unsupported);
        }
        if input.read_u8()? /* Version */ != 1 {
            return Err(Error::Unsupported);
        }
        let c_key_size = input.read_u8()?;
        let e_key_size = input.read_u8()?;
        let c_to_e_key_page_size = input.read_u16::<BigEndian>()?;
        let e_key_spec_page_size = input.read_u16::<BigEndian>()?;
        let c_to_e_key_page_count = input.read_u32::<BigEndian>()?;
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

        let mut c_to_e_key_page_table = Vec::with_capacity(c_to_e_key_page_count as usize);
        for _ in 0..c_to_e_key_page_count {
            c_to_e_key_page_table.push(EncodingPage::decode(input, c_key_size)?);
        }
        let mut c_to_e_keys = HashMap::new();
        for c_to_e_key_page in &mut c_to_e_key_page_table {
            let mut c_to_e_key_page_data = vec![0; c_to_e_key_page_size as usize * 1024];
            input.read_exact(&mut c_to_e_key_page_data)?;
            let mut c_to_e_key_page_md5 = Md5::new();
            c_to_e_key_page_md5.update(&c_to_e_key_page_data);
            if c_to_e_key_page.md5 != c_to_e_key_page_md5.finalize().as_slice() {
                return Err(Error::IntegrityError);
            }
            let mut c_to_e_key_page_data = c_to_e_key_page_data.as_slice();
            while let Ok(c_to_e_key) =
                EncodingCToEKey::decode(&mut c_to_e_key_page_data, c_key_size, e_key_size)
            {
                c_to_e_keys.insert(c_to_e_key.c_key.clone(), c_to_e_key);
            }
        }

        let mut e_key_spec_page_table = Vec::with_capacity(e_key_spec_page_count as usize);
        for _ in 0..e_key_spec_page_count {
            e_key_spec_page_table.push(EncodingPage::decode(input, e_key_size)?);
        }
        let mut e_key_specs = HashMap::new();
        for e_key_spec_page in &mut e_key_spec_page_table {
            let mut e_key_spec_page_data = vec![0; e_key_spec_page_size as usize * 1024];
            input.read_exact(&mut e_key_spec_page_data)?;
            let mut e_key_spec_page_md5 = Md5::new();
            e_key_spec_page_md5.update(&e_key_spec_page_data);
            if e_key_spec_page.md5 != e_key_spec_page_md5.finalize().as_slice() {
                return Err(Error::IntegrityError);
            }
            let mut e_key_spec_page_data = e_key_spec_page_data.as_slice();
            while let Ok(e_key_spec) =
                EncodingEKeySpec::decode(&mut e_key_spec_page_data, e_key_size, &e_specs)
            {
                e_key_specs.insert(e_key_spec.e_key.clone(), e_key_spec);
            }
        }

        Ok(Self {
            c_to_e_keys,
            e_key_specs,
        })
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.c_to_e_keys
            .get(key)
            .map(|c_to_e_key| c_to_e_key.e_keys.first().unwrap().clone())
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

#[derive(Debug)]
struct EncodingCToEKey {
    c_key: Vec<u8>,
    c_size: u64,
    e_keys: Vec<Vec<u8>>,
}

impl EncodingCToEKey {
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

#[derive(Debug)]
struct EncodingEKeySpec {
    e_key: Vec<u8>,
    e_size: u64,
    e_spec: String,
}

impl EncodingEKeySpec {
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

#[inline]
fn read_asciiz<R: Read>(input: &mut R) -> Result<String> {
    let mut data = Vec::new();
    loop {
        let value = input.read_u8()?;
        if value == 0u8 {
            break;
        }
        data.push(value as char);
    }

    Ok(data.iter().collect())
}
