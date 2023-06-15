use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

pub use encoding::Encoding;

use crate::{Error, md5, read_asciiz, Result};

mod encoding;

#[derive(Debug)]
pub struct BuildInfo {
    pub build_config: u128,
}

impl BuildInfo {
    pub fn parse<R: Read>(input: &mut R) -> Result<Vec<Self>> {
        let mut header = true;
        let mut rows = Vec::new();
        for row in BufReader::new(input).lines() {
            let row = row.unwrap();
            if header {
                header = false;
                continue;
            }

            let mut columns = row.split('|');
            columns.next().unwrap();
            columns.next().unwrap();
            rows.push(BuildInfo {
                build_config: u128::from_str_radix(columns.next().unwrap(), 16).unwrap(),
            });
        }

        Ok(rows)
    }
}

#[derive(Debug)]
pub struct BuildConfig {
    pub root: Record,
    pub encoding: Record,
}

impl BuildConfig {
    pub fn parse<R: Read>(input: &mut R) -> Result<Self> {
        let mut entries = HashMap::new();
        for line in BufReader::new(input).lines() {
            let line = line.unwrap();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut entry = line.split(" = ");
            entries.insert(
                entry.next().unwrap().to_owned(),
                entry.next().unwrap().to_owned(),
            );
        }
        Ok(Self {
            root: Record::parse(&entries["root"]),
            encoding: Record::parse(&entries["encoding"]),
        })
    }
}

#[derive(Debug)]
pub struct Record {
    pub c_key: u128,
    pub e_key: Option<u128>,
}

impl Record {
    pub fn parse(value: &str) -> Self {
        let mut values = value.split(' ');
        Self {
            c_key: u128::from_str_radix(values.next().unwrap(), 16).unwrap(),
            e_key: values
                .next()
                .map(|value| u128::from_str_radix(value, 16).unwrap()),
        }
    }
}

#[derive(Debug)]
pub struct RootFile {
    pub id: String,
    pub md5: u128,
}

impl RootFile {
    pub fn parse<R: Read>(input: &mut R) -> Result<Vec<Self>> {
        let mut rows = Vec::new();
        for row in BufReader::new(input).lines() {
            let row = row.unwrap();
            if row.is_empty() || row.starts_with('#') {
                continue;
            }

            let mut columns = row.split('|');
            rows.push(RootFile {
                id: columns.next().unwrap().to_owned(),
                md5: u128::from_str_radix(columns.next().unwrap(), 16).unwrap(),
            });
        }

        Ok(rows)
    }
}

#[derive(Debug)]
pub struct Index {
    pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
    pub key: u128,
    pub offset: u32,
    pub length: u32,
}

impl Index {
    pub fn decode<R: Read + Seek>(input: &mut R) -> Result<Self> {
        let block_size = 0x1000;
        let block_count = input.stream_len()? / block_size;
        input.seek(SeekFrom::Start(block_count * block_size))?;
        let mut _block_data = vec![0; (input.stream_len()? - block_count * block_size) as usize];
        input.read_exact(&mut _block_data)?;
        let mut block_data = _block_data.as_slice();

        // table of content
        for _ in 0..block_count {
            let _last_key = block_data.read_u128::<BigEndian>()?;
        }
        for _ in 0..block_count {
            let _checksum = block_data.read_u64::<BigEndian>()?;
        }

        // footer
        let toc_checksum = block_data.read_u64::<BigEndian>()?;
        if toc_checksum != md5(&_block_data[..(block_count * (16 + 8)) as usize]) as u64 {
            return Err(Error::IntegrityError);
        }
        let version = block_data.read_u8()?;
        if version != 1 {
            return Err(Error::Unsupported);
        }
        if block_data.read_u8()? != 0 {
            return Err(Error::Unsupported);
        }
        if block_data.read_u8()? != 0 {
            return Err(Error::Unsupported);
        }
        let block_size = block_data.read_u8()?;
        if block_size != 4 {
            return Err(Error::Unsupported);
        }
        let offset_size = block_data.read_u8()?;
        if offset_size != 4 {
            return Err(Error::Unsupported);
        }
        let length_size = block_data.read_u8()?;
        if length_size != 4 {
            return Err(Error::Unsupported);
        }
        let key_size = block_data.read_u8()?;
        if key_size != 16 {
            return Err(Error::Unsupported);
        }
        let checksum_size = block_data.read_u8()?;
        if checksum_size != 8 {
            return Err(Error::Unsupported);
        }
        let entry_count = block_data.read_u32::<LittleEndian>()?;
        let _footer_checksum = block_data.read_u64::<BigEndian>()?;

        // entries
        input.seek(SeekFrom::Start(0))?;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..block_count {
            let mut block_data = vec![0; block_size as usize];
            input.read_exact(&mut block_data)?;
            let mut block_data = block_data.as_slice();
            for _ in 0..block_size / (16 + 4 + 4) {
                entries.push(Entry {
                    key: block_data.read_u128::<BigEndian>()?,
                    offset: block_data.read_u32::<BigEndian>()?,
                    length: block_data.read_u32::<BigEndian>()?,
                });
            }
        }

        Ok(Index { entries })
    }
}

#[derive(Debug)]
pub struct Install {
    pub tags: Vec<InstallTag>,
}

#[derive(Debug)]
pub struct InstallTag {
    pub name: String,
    pub kind: u16,
    pub files: Vec<InstallEntry>,
}

#[derive(Debug)]
pub struct InstallEntry {
    pub name: String,
    pub hash: u128,
    pub length: u32,
}

impl Install {
    pub fn decode<R: Read>(input: &mut R) -> Result<Self> {
        if input.read_u16::<BigEndian>()? != u16::from_be_bytes(*b"IN") {
            return Err(Error::Unsupported);
        }
        let version = input.read_u8()?;
        if version != 1 {
            return Err(Error::Unsupported);
        }
        let hash_size = input.read_u8()?;
        if hash_size != 16 {
            return Err(Error::Unsupported);
        }
        let tag_count = input.read_u16::<BigEndian>()?;
        let entry_count = input.read_u32::<BigEndian>()?;
        for _ in 0..tag_count {
            InstallTag {
                name: read_asciiz(input)?,
                kind: input.read_u16::<BigEndian>()?,
                files: vec![],
            };
            // mask
        }
        for _ in 0..entry_count {
            InstallEntry {
                name: read_asciiz(input)?,
                hash: input.read_u128::<BigEndian>()?,
                length: input.read_u32::<BigEndian>()?,
            };
        }
        Ok(Install { tags: vec![] })
    }
}

pub struct Download {}

impl Download {
    pub fn decode<R: Read>(input: &mut R) -> Result<Self> {
        if input.read_u16::<BigEndian>()? != u16::from_be_bytes(*b"DL") {
            return Err(Error::Unsupported);
        }
        let version = input.read_u8()?;
        if version != 1 {
            return Err(Error::Unsupported);
        }
        let key_size = input.read_u8()?;
        if key_size != 16 {
            return Err(Error::Unsupported);
        }
        input.read_u8()?;
        let entry_count = input.read_u32::<BigEndian>()?;
        let tag_count = input.read_u16::<BigEndian>()?;
        for _ in 0..entry_count {
            input.read_u128::<BigEndian>()?;
            input.read_uint::<BigEndian>(5)?;
            input.read_u8()?;
            input.read_u32::<BigEndian>()?;
        }
        for _ in 0..tag_count {
            read_asciiz(input)?;
            input.read_u16::<BigEndian>()?;
            // mask
        }
        Ok(Download {})
    }
}
