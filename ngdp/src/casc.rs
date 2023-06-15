use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use fasthash::lookup3;

use crate::{blte, Error, Result};

pub struct Storage {
    path: PathBuf,
    entries: HashMap<u128, Entry>,
}

impl Storage {
    pub fn new(_path: impl AsRef<Path>) -> Result<Self> {
        let mut path = PathBuf::new();
        path.push(_path);

        let shared_memory = SharedMemory::decode(&mut File::open(path.join("shmem"))?, 0x10)?;
        let mut entries = HashMap::new();
        for (bucket, version) in shared_memory.versions.into_iter().enumerate() {
            let mut index_file = File::open(path.join(format!("{bucket:02x}{version:08x}.idx")))?;
            let index = Index::decode(&mut index_file)?;
            for entry in index.entries {
                entries.insert(entry.key, entry);
            }
        }

        Ok(Self { path, entries })
    }

    pub fn get(&self, key: u128) -> Result<Option<Vec<u8>>> {
        let Some(entry) = self.entries.get(&(key >> 56)) else {
            return Ok(None);
        };
        let mut file = File::open(self.path.join(format!("data.{:03}", entry.file)))?;
        entry.decode_data_header(&mut file)?;
        Ok(Some(blte::decode(&mut file, |_key_name| None)?))
    }
}

struct SharedMemory {
    path: String,
    versions: Vec<u32>,
}

impl SharedMemory {
    fn decode<R: Read + Seek>(input: &mut R, bucket_count: usize) -> Result<Self> {
        // header block
        let block_type = input.read_u32::<LittleEndian>()?;
        if block_type != 4 && block_type != 5 {
            return Err(Error::Unsupported);
        }
        let block_size = input.read_u32::<LittleEndian>()?;
        let mut path = vec![0; 0x100];
        input.read_exact(&mut path)?;
        for _ in 0..(block_size - 4 - 4 - 0x100 - bucket_count as u32 * 4) / (2 * 4) {
            let _block_size = input.read_u32::<LittleEndian>()?;
            let _block_offset = input.read_u32::<LittleEndian>()?;
        }
        let mut versions = Vec::with_capacity(bucket_count);
        for _ in 0..bucket_count {
            versions.push(input.read_u32::<LittleEndian>()?);
        }

        // free space block
        /*if input.read_u32::<LittleEndian>()? != 1 {
            return Err(Error::Unsupported);
        }
        let block_size = input.read_u32::<LittleEndian>()?;
        let mut free_spaces = Vec::with_capacity(block_size as usize);
        input.seek(SeekFrom::Current(0x18))?;
        for _ in 0..block_size {
            let length = Entry::decode(input, 0, 5, 0, 30)?;
            free_spaces.push(Entry {
                key: vec![],
                file: 0,
                offset: 0,
                length: length.offset,
            });
        }
        input.seek(SeekFrom::Current(((1090 - block_size) * 5) as i64))?;
        for index in 0..block_size {
            let file_offset = Entry::decode(input, 0, 5, 0, 30)?;
            let entry = &mut free_spaces[index as usize];
            entry.file = file_offset.file;
            entry.offset = file_offset.offset;
        }*/

        Ok(Self {
            path: std::str::from_utf8(
                &path[0..path
                    .as_slice()
                    .iter()
                    .position(|&value| value == b'\0')
                    .unwrap_or(path.len())],
            )?
            .to_string(),
            versions,
        })
    }
}

struct Index {
    bucket: u16,
    entry_length_size: u8,
    entry_location_size: u8,
    entry_key_size: u8,
    entry_segment_bits: u8,
    limit: u64,
    entries: Vec<Entry>,
}

impl Index {
    fn decode<R: Read>(input: &mut R) -> Result<Self> {
        // header
        let mut header_data = vec![0; input.read_u32::<LittleEndian>()? as usize];
        let header_hash = input.read_u32::<LittleEndian>()?;
        input.read_exact(&mut header_data)?;
        let mut header_data = header_data.as_slice();
        if header_hash != lookup3::hash32(header_data) {
            return Err(Error::IntegrityError);
        }
        if header_data.read_u16::<LittleEndian>()? != 7 {
            return Err(Error::Unsupported);
        }
        let bucket = header_data.read_u16::<LittleEndian>()?;
        let entry_length_size = header_data.read_u8()?;
        let entry_location_size = header_data.read_u8()?;
        let entry_key_size = header_data.read_u8()?;
        let entry_segment_bits = header_data.read_u8()?;
        let limit = header_data.read_u64::<LittleEndian>()?;
        input.read_exact(&mut [0; 0x8])?; // padding

        // entries
        let mut entries_data = vec![0; input.read_u32::<LittleEndian>()? as usize];
        let _entries_hash = input.read_u32::<LittleEndian>()?;
        input.read_exact(&mut entries_data)?;
        let mut entries_data = entries_data.as_slice();
        /*if entries_hash != lookup3::hash32(entries_data) {
            return Err(Error::IntegrityError);
        }*/
        let entry_count = entries_data.len()
            / (entry_length_size + entry_location_size + entry_key_size) as usize;
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(Entry::decode(
                &mut entries_data,
                entry_length_size,
                entry_location_size,
                entry_key_size,
                entry_segment_bits,
            )?);
        }

        Ok(Self {
            bucket,
            entry_length_size,
            entry_location_size,
            entry_key_size,
            entry_segment_bits,
            limit,
            entries,
        })
    }
}

struct Entry {
    key: u128,
    file: u64,
    offset: u64,
    length: u64,
}

impl Entry {
    fn decode<R: Read>(
        input: &mut R,
        length_size: u8,
        location_size: u8,
        key_size: u8,
        segment_bits: u8,
    ) -> Result<Self> {
        let key = input.read_uint128::<BigEndian>(key_size as usize)?;
        let offset_size = (segment_bits + 7) / 8;
        let file_size = location_size - offset_size;
        let mut file = input.read_uint::<BigEndian>(file_size as usize)?;
        let mut offset = input.read_uint::<BigEndian>(offset_size as usize)?;
        let extra_bits = (offset_size * 8) - segment_bits;
        file = (file << extra_bits) | (offset >> segment_bits);
        offset &= (1 << (32 - extra_bits)) - 1;
        let length = if length_size == 0 {
            0
        } else {
            input.read_uint::<LittleEndian>(length_size as usize)?
        };
        Ok(Self {
            key,
            file,
            offset,
            length,
        })
    }

    fn decode_data_header<R: Read + Seek>(&self, input: &mut R) -> Result<()> {
        input.seek(SeekFrom::Start(self.offset))?;
        let mut key = [0; 0x10];
        input.read_exact(&mut key)?;
        if input.read_u32::<LittleEndian>()? != self.length as u32 {
            return Err(Error::IntegrityError);
        }
        input.read_u16::<LittleEndian>()?;
        {
            let checksum_offset = input.stream_position()?;
            input.seek(SeekFrom::Start(self.offset))?;
            let mut data = vec![0; (checksum_offset - self.offset) as usize];
            input.read_exact(&mut data)?;
            if input.read_u32::<LittleEndian>()? != lookup3::hash32_with_seed(data, 0x3D6BE971) {
                return Err(Error::IntegrityError);
            }
        }
        {
            let encoded_offset = ((self.offset & 0x3FFFFFFF) | (self.file & 3) << 30) as u32;
            let encoded_checksum_offset =
                ((input.stream_position()? & 0x3FFFFFFF) | (self.file & 3) << 30) as u32;
            input.seek(SeekFrom::Start(self.offset))?;
            let mut hashed_data = [0u8; 4];
            for i in encoded_offset..encoded_checksum_offset {
                hashed_data[(i & 3) as usize] ^= input.read_u8()?;
            }
            let encoded_offset: [u8; 4] = (OFFSET_ENCODE_TABLE
                [((encoded_checksum_offset + 4) & 0xF) as usize]
                ^ (encoded_checksum_offset + 4))
                .to_ne_bytes();
            let checksum: [_; 4] = core::array::from_fn(|i| {
                let j = (i + encoded_checksum_offset as usize) & 3;
                hashed_data[j] ^ encoded_offset[j]
            });
            let checksum: u32 = unsafe { std::mem::transmute(checksum) };
            if input.read_u32::<LittleEndian>()? != checksum {
                return Err(Error::IntegrityError);
            }
        }
        Ok(())
    }
}

static OFFSET_ENCODE_TABLE: &[u32] = &[
    0x049396B8, 0x72A82A9B, 0xEE626CCA, 0x9917754F, 0x15DE40B1, 0xF5A8A9B6, 0x421EAC7E, 0xA9D55C9A,
    0x317FD40C, 0x04FAF80D, 0x3D6BE971, 0x52933CFD, 0x27F64B7D, 0xC6F5C11B, 0xD5757E3A, 0x6C388745,
];
