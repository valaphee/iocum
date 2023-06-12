use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use md5::{Digest, Md5};
use salsa20::{
    cipher::{KeyIvInit, StreamCipher},
    Salsa20,
};

use crate::{Error, Result};

pub fn decode<'a, R: Read>(
    input: &mut R,
    key_lookup: impl Fn(&[u8]) -> Option<&'a [u8]>,
) -> Result<Vec<u8>> {
    if input.read_u32::<BigEndian>()? != u32::from_be_bytes(*b"BLTE") {
        return Err(Error::Unsupported);
    }
    let header_size = input.read_u32::<BigEndian>()?;
    if input.read_u8()? != 0xF {
        return Err(Error::Unsupported);
    }
    let chunk_count = input.read_u24::<BigEndian>()?;
    if header_size != 4 + 4 + 1 + 3 + chunk_count * (4 + 4 + 16) {
        return Err(Error::Unsupported);
    }
    let mut chunks = Vec::with_capacity(chunk_count as usize);
    for _ in 0..chunk_count {
        chunks.push(Chunk::decode(input)?);
    }

    let mut content =
        Vec::with_capacity(chunks.iter().map(|chunk| chunk.c_size).sum::<u32>() as usize);
    for chunk in chunks.iter() {
        let mut encoded = vec![0; chunk.e_size as usize];
        input.read_exact(&mut encoded)?;
        let mut encoded = encoded.as_slice();
        let mut md5 = Md5::new();
        md5.update(encoded);
        if chunk.md5 != md5.finalize().as_slice() {
            return Err(Error::IntegrityError);
        }
        let encoding_mode = encoded.read_u8()?;
        match encoding_mode {
            b'N' => {
                content.extend(encoded);
            }
            b'Z' => {
                ZlibDecoder::new(encoded).read_to_end(&mut content)?;
            }
            b'E' => {
                let key_name_size = encoded.read_u8()?;
                let mut key_name = vec![0; key_name_size as usize];
                encoded.read_exact(&mut key_name)?;
                let Some(key) = key_lookup(&key_name) else {
                    return Err(Error::KeyNotFound(hex::encode(key_name)));
                };
                let iv_size = encoded.read_u8()?;
                let mut iv = vec![0; iv_size as usize];
                encoded.read_exact(&mut iv)?;
                let encryption_type = encoded.read_u8()?;
                match encryption_type {
                    b'S' => {
                        Salsa20::new(key.into(), iv.as_slice().into())
                            .apply_keystream(&mut content[..encoded.len()]);
                        content.write_all(encoded)?;
                    }
                    _ => {
                        return Err(Error::UnknownEncryptionMode(encryption_type as char));
                    }
                }
            }
            _ => {
                return Err(Error::UnknownEncodingMode(encoding_mode as char));
            }
        }
    }

    Ok(content)
}

struct Chunk {
    e_size: u32,
    c_size: u32,
    md5: [u8; 16],
}

impl Chunk {
    fn decode<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            e_size: input.read_u32::<BigEndian>()?,
            c_size: input.read_u32::<BigEndian>()?,
            md5: {
                let mut md5 = [0; 16];
                input.read_exact(&mut md5)?;
                md5
            },
        })
    }
}
