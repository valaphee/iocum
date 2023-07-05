use std::{
    collections::HashMap,
    fs::File,
    hash::Hasher,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
};

use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;

macro_rules! hasher_to_fcn {
    ($(#[$attr:meta])* $name:ident, $hasher:ident) => {
        $(#[$attr])*
        #[inline]
        pub fn $name(bytes: &[u8]) -> u64 {
            let mut hasher = $hasher::default();
            hasher.write(bytes);
            hasher.finish()
        }
    };
}

#[derive(Default)]
pub struct FNV0Hasher32(u32);

impl Hasher for FNV0Hasher32 {
    #[inline]
    fn finish(&self) -> u64 {
        self.0 as u64
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes.iter() {
            self.0 = self.0.wrapping_mul(0x01000193);
            self.0 = self.0 ^ (*byte as u32);
        }
    }
}

hasher_to_fcn!(fnv0, FNV0Hasher32);

pub struct FileList {
    pub entries: Vec<FileEntry>,
}

impl FileList {
    pub fn parse(data: &mut impl Read) -> Self {
        let data = BufReader::new(data);
        let mut lines = data.lines();
        let mut entries =
            Vec::with_capacity(lines.next().unwrap().unwrap().parse::<u32>().unwrap() as usize);
        for line in lines {
            let line = line.unwrap();
            let mut line = line.split(',');
            entries.push(FileEntry {
                file_name: line.next().unwrap().to_owned(),
                path_name: line.next().unwrap().to_owned(),
                time: u32::from_str_radix(line.next().unwrap(), 16).unwrap(),
                compressed_size: line.next().unwrap().parse::<u32>().unwrap(),
                size: line.next().unwrap().parse::<u32>().unwrap(),
                compressed_checksum: u32::from_str_radix(line.next().unwrap(), 16).unwrap(),
                checksum: u32::from_str_radix(line.next().unwrap(), 16).unwrap(),
            });
        }
        Self { entries }
    }
}

pub struct FileEntry {
    pub file_name: String,
    pub path_name: String,
    pub time: u32,
    pub compressed_size: u32,
    pub size: u32,
    pub compressed_checksum: u32,
    pub checksum: u32,
}

#[derive(Debug)]
pub struct Index {
    pub entries: Vec<IndexEntry>,
}

impl Index {
    pub fn decode(data: &mut impl Read) -> Self {
        if data.read_u32::<LittleEndian>().unwrap() != 0x20151018 {
            todo!("")
        }
        let mut entries = vec![];
        while let Ok(hash) = data.read_u32::<LittleEndian>() {
            entries.push(IndexEntry {
                hash,
                offset: data.read_u32::<LittleEndian>().unwrap() ^ hash,
                size: data.read_u32::<LittleEndian>().unwrap() ^ hash,
                checksum: data.read_u32::<LittleEndian>().unwrap(),
                time: data.read_u32::<LittleEndian>().unwrap(),
            });
        }
        Index { entries }
    }
}

#[derive(Debug)]
pub struct IndexEntry {
    pub hash: u32,
    pub offset: u32,
    pub size: u32,
    pub checksum: u32,
    pub time: u32,
}

fn main() {
    let file_list = FileList::parse(
        &mut File::open(r#"C:\Users\valaphee\Downloads\LAOnline\FileList.txt"#).unwrap(),
    );
    let index = Index::decode(
        &mut File::open(r#"C:\Users\valaphee\Downloads\LAOnline\nfs\nfs.idx"#).unwrap(),
    );
    let mut entry_offsets = HashMap::new();
    for entry in index.entries {
        entry_offsets.insert(entry.hash, entry.offset);
    }
    for entry in file_list.entries {
        if entry.file_name.len() != 8 {
            continue;
        }
        let Ok(file_name) = u32::from_str_radix(&entry.file_name, 16) else {
            continue;
        };
        let offset = *entry_offsets.get(&file_name).unwrap();

        let Ok(mut archive) = File::open(format!(r#"C:\Users\valaphee\Downloads\LAOnline\nfs\{}\{}"#, entry.path_name.chars().next().unwrap(), entry.path_name)) else {
            continue;
        };
        archive.seek(SeekFrom::Start(offset as u64)).unwrap();

        let hash = archive.read_u32::<LittleEndian>().unwrap();
        let _checksum = archive.read_u32::<LittleEndian>().unwrap();
        let _size = archive.read_u32::<LittleEndian>().unwrap() ^ hash;
        let mut data = vec![];
        ZlibDecoder::new(archive).read_to_end(&mut data).unwrap();

        std::fs::create_dir_all(format!(
            r#"C:\Users\valaphee\Downloads\LAOnline\nfs\{}\{}-extracted"#,
            entry.path_name.chars().next().unwrap(),
            entry.path_name
        ))
        .unwrap();
        let file_extension = if data.starts_with(b"DDS ") {
            ".dds"
        } else if data.starts_with(b"Gamebryo File Format, Version 20.2.0.8") {
            ".nif"
        } else if data.starts_with(b"BULLET") {
            ".bullet"
        } else if data.starts_with(b";Gamebryo KFM File Version 2.2.0.0b") {
            ".kfm"
        } else {
            ""
        };
        std::fs::write(
            format!(
                r#"C:\Users\valaphee\Downloads\LAOnline\nfs\{}\{}-extracted\{}{}"#,
                entry.path_name.chars().next().unwrap(),
                entry.path_name,
                entry.file_name,
                file_extension
            ),
            data,
        )
        .unwrap();
    }
}
