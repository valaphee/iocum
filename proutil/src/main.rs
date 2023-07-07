extern crate core;

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use byteorder::{ReadBytesExt, LE};
use object::{read::pe::PeFile64, LittleEndian};

fn main() {
    let mut file = File::open(r#""#).unwrap();
    let mut data = vec![];
    file.read_to_end(&mut data).unwrap();
    let image = PeFile64::parse(data.as_slice()).unwrap();
    let section = image
        .section_table()
        .iter()
        .find(|section| section.name.as_slice() == b".data\0\0\0")
        .unwrap();
    let start = section.pointer_to_raw_data.get(LittleEndian) as usize;
    let end = start + section.size_of_raw_data.get(LittleEndian) as usize;
    let mut i = start;
    loop {
        if i >= end {
            break;
        }
        // encrypted?
        if data[i] != 1 {
            i += 1;
            continue;
        }
        // ignore empty strings
        let length = (data[i - 3] as u32
            | ((data[i - 2] as u32) << 8)
            | ((data[i - 1] as u32) << 16)) as usize;
        if length == 0 {
            i += 1;
            continue;
        }
        // string is longer than total length
        if i + 1 + length >= data.len() {
            i += 1;
            continue;
        }
        // string seem to be still zero-terminated
        if data[i + 1 + length] != 0 {
            i += 1;
            continue;
        }
        // xor
        for j in 0..length {
            data[i + 1 + j] ^= data[i - 3 - 8 + (j % 8)];
        }
        // try decode
        let Ok(string) = std::str::from_utf8(&data[i + 1..i + 1 + length]) else {
            // undo xor
            for j in 0..length {
                data[i + 1 + j] ^= data[i - 3 - 8 + (j % 8)];
            }
            i += 1;
            continue;
        };
        // check if string contains non whitespace control characters
        if string.contains(|value: char| value.is_ascii_control() && !value.is_ascii_whitespace()) {
            // undo xor
            for j in 0..length {
                data[i + 1 + j] ^= data[i - 3 - 8 + (j % 8)];
            }
            i += 1;
            continue;
        }
        println!("{:x} {}", i, string);
        data[i] = 0;
        i += length;
    }
    std::fs::write(r#""#, data.as_slice()).unwrap();
}
