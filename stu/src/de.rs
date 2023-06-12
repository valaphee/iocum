use byteorder::{LittleEndian, ReadBytesExt};

use crate::{
    crc64,
    error::{Error, Result},
    DynamicDataHeader, Field, Header, InlineArray, Instance, CRC32,
};

pub struct Deserializer<'de> {
    data: &'de [u8],
    dynamic_data: &'de [u8],

    header_crc: u64,
    instances: &'de [Instance],
    inline_arrays: &'de [InlineArray],
    field_sets: Vec<&'de [Field]>,

    current_field_hash: u32,
    current_field_size: u32,
    dynamic_data_slice: Option<&'de [u8]>,
}

impl<'de> Deserializer<'de> {
    pub fn from_slice(data: &'de mut [u8]) -> Result<Self> {
        let header_end = std::mem::size_of::<Header>();
        if data.len() < header_end {
            return Err(Error::InvalidData);
        }
        let header: &Header = bytemuck::from_bytes(&data[..header_end]);
        let header_crc = crc64(&data[..std::mem::size_of::<Header>()]);

        let instances_end = header.instance_offset as usize
            + header.instance_count as usize * std::mem::size_of::<Instance>();
        if data.len() < instances_end {
            return Err(Error::InvalidData);
        }
        let instances: &[Instance] =
            bytemuck::cast_slice(&data[header.instance_offset as usize..instances_end]);

        let inline_arrays_end = header.inline_array_offset as usize
            + header.inline_array_count as usize * std::mem::size_of::<Instance>();
        if data.len() < inline_arrays_end {
            return Err(Error::InvalidData);
        }
        let inline_arrays: &[InlineArray] =
            bytemuck::cast_slice(&data[header.inline_array_offset as usize..inline_arrays_end]);

        let field_bags_end = header.field_set_offset as usize + header.field_set_count as usize * 8;
        if data.len() < field_bags_end {
            return Err(Error::InvalidData);
        }
        let mut field_bags: Vec<&[Field]> = Vec::new();
        {
            let mut field_bags_data = &data[header.field_set_offset as usize..field_bags_end];
            for _ in 0..header.field_set_count {
                let count = field_bags_data.read_u32::<LittleEndian>()? as usize;
                let offset = field_bags_data.read_u32::<LittleEndian>()? as usize;
                let field_bag_end = offset + count * std::mem::size_of::<Field>();
                if data.len() < field_bag_end {
                    return Err(Error::InvalidData);
                }
                field_bags.push(bytemuck::cast_slice(&data[offset..field_bag_end]))
            }
        }

        if data.len() < header.data_offset as usize {
            return Err(Error::InvalidData);
        }
        let dynamic_data_end =
            header.dynamic_data_offset as usize + header.dynamic_data_size as usize;
        if data.len() < dynamic_data_end {
            return Err(Error::InvalidData);
        }

        Ok(Self {
            data: &data[header.data_offset as usize..instances.first().unwrap().size as usize],
            dynamic_data: &data[header.dynamic_data_offset as usize..dynamic_data_end],

            instances,
            inline_arrays,
            field_sets: field_bags,
            header_crc,

            current_field_hash: 0,
            current_field_size: 0,
            dynamic_data_slice: None,
        })
    }
}

impl<'de, 'a> serde::de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i8(
            if let Some(dynamic_data_slice) = &mut self.dynamic_data_slice {
                dynamic_data_slice.read_i8()
            } else {
                if self.current_field_size != 1 {
                    return Err(Error::InvalidLength {
                        length: self.current_field_size,
                        expected: 1,
                    });
                }

                self.data.read_i8()
            }?,
        )
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i16(
            if let Some(dynamic_data_slice) = &mut self.dynamic_data_slice {
                dynamic_data_slice.read_i16::<LittleEndian>()
            } else {
                if self.current_field_size != 2 {
                    return Err(Error::InvalidLength {
                        length: self.current_field_size,
                        expected: 2,
                    });
                }

                self.data.read_i16::<LittleEndian>()
            }?,
        )
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i32(
            if let Some(dynamic_data_slice) = &mut self.dynamic_data_slice {
                dynamic_data_slice.read_i32::<LittleEndian>()
            } else {
                if self.current_field_size != 4 {
                    return Err(Error::InvalidLength {
                        length: self.current_field_size,
                        expected: 4,
                    });
                }

                self.data.read_i32::<LittleEndian>()
            }?,
        )
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i64(
            if let Some(dynamic_data_slice) = &mut self.dynamic_data_slice {
                dynamic_data_slice.read_i64::<LittleEndian>()
            } else {
                if self.current_field_size != 8 {
                    return Err(Error::InvalidLength {
                        length: self.current_field_size,
                        expected: 8,
                    });
                }

                self.data.read_i64::<LittleEndian>()
            }?,
        )
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u8(
            if let Some(dynamic_data_slice) = &mut self.dynamic_data_slice {
                dynamic_data_slice.read_u8()
            } else {
                if self.current_field_size != 1 {
                    return Err(Error::InvalidLength {
                        length: self.current_field_size,
                        expected: 1,
                    });
                }

                self.data.read_u8()
            }?,
        )
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u16(
            if let Some(dynamic_data_slice) = &mut self.dynamic_data_slice {
                dynamic_data_slice.read_u16::<LittleEndian>()
            } else {
                if self.current_field_size != 2 {
                    return Err(Error::InvalidLength {
                        length: self.current_field_size,
                        expected: 2,
                    });
                }

                self.data.read_u16::<LittleEndian>()
            }?,
        )
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u32(
            if let Some(dynamic_data_slice) = &mut self.dynamic_data_slice {
                dynamic_data_slice.read_u32::<LittleEndian>()
            } else {
                if self.current_field_size != 4 {
                    return Err(Error::InvalidLength {
                        length: self.current_field_size,
                        expected: 4,
                    });
                }

                self.data.read_u32::<LittleEndian>()
            }?,
        )
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut value = if let Some(dynamic_data_slice) = &mut self.dynamic_data_slice {
            dynamic_data_slice.read_u64::<LittleEndian>()
        } else {
            if self.current_field_size != 8 {
                return Err(Error::InvalidLength {
                    length: self.current_field_size,
                    expected: 8,
                });
            }

            self.data.read_u64::<LittleEndian>()
        }?;

        // likely to be an obfuscated GUID
        value ^= (self.current_field_hash as u64 | ((self.current_field_hash as u64) << 32))
            ^ self.header_crc;
        let mut bytes = value.to_le_bytes();
        bytes.swap(0, 3);
        bytes.swap(7, 1);
        bytes.swap(2, 6);
        bytes.swap(4, 5);
        visitor.visit_u64(u64::from_le_bytes(bytes))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f32(
            if let Some(dynamic_data_slice) = &mut self.dynamic_data_slice {
                dynamic_data_slice.read_f32::<LittleEndian>()
            } else {
                if self.current_field_size != 4 {
                    return Err(Error::InvalidLength {
                        length: self.current_field_size,
                        expected: 4,
                    });
                }

                self.data.read_f32::<LittleEndian>()
            }?,
        )
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f64(
            if let Some(dynamic_data_slice) = &mut self.dynamic_data_slice {
                dynamic_data_slice.read_f64::<LittleEndian>()
            } else {
                if self.current_field_size != 8 {
                    return Err(Error::InvalidLength {
                        length: self.current_field_size,
                        expected: 8,
                    });
                }

                self.data.read_f64::<LittleEndian>()
            }?,
        )
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 4 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 4,
            });
        }

        let offset = self.data.read_u32::<LittleEndian>()? as usize;
        let dynamic_data_header: &DynamicDataHeader = bytemuck::from_bytes(
            &self.dynamic_data[offset..offset + std::mem::size_of::<DynamicDataHeader>()],
        );
        let dynamic_data = &self.dynamic_data[dynamic_data_header.offset as usize
            ..dynamic_data_header.offset as usize + dynamic_data_header.size as usize];

        let mut digest = CRC32.digest();
        digest.update(dynamic_data);
        if dynamic_data_header.crc != digest.finalize() {
            return Err(Error::IntegrityError);
        }

        visitor.visit_str(std::str::from_utf8(dynamic_data).unwrap())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 4 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 4,
            });
        }

        let offset = self.data.read_u32::<LittleEndian>()? as usize;
        let dynamic_data_header: &DynamicDataHeader = bytemuck::from_bytes(
            &self.dynamic_data[offset..offset + std::mem::size_of::<DynamicDataHeader>()],
        );
        let dynamic_data = &self.dynamic_data[dynamic_data_header.offset as usize
            ..dynamic_data_header.offset as usize + dynamic_data_header.size as usize];

        let mut digest = CRC32.digest();
        digest.update(dynamic_data);
        if dynamic_data_header.crc != digest.finalize() {
            return Err(Error::IntegrityError);
        }

        visitor.visit_string(std::str::from_utf8(dynamic_data).unwrap().to_string())
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        // this is only needed for support reading optional fields
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let count = if self.current_field_size != 4 {
            self.data.read_u32::<LittleEndian>()?
        } else {
            let offset = self.data.read_u32::<LittleEndian>()? as usize;
            let dynamic_data_header: &DynamicDataHeader = bytemuck::from_bytes(
                &self.dynamic_data[offset..offset + std::mem::size_of::<DynamicDataHeader>()],
            );
            self.dynamic_data_slice =
                Some(&self.dynamic_data[dynamic_data_header.offset as usize..]);

            /*let mut digest = CRC32.digest();
            digest.update(self.dynamic_data_slice.unwrap());
            if dynamic_data_header.crc != digest.finalize() {
                return Err(Error::IntegrityError);
            }*/

            dynamic_data_header.size
        };

        visitor.visit_seq(SeqAccess { de: self, count })
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let field_set_id = self.data.read_u32::<LittleEndian>()?;
        let fields = self.field_sets.get(field_set_id as usize).unwrap().iter();
        visitor.visit_map(MapAccess { de: self, fields })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_str(&format!("{:08X}", self.current_field_hash))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.data = &self.data[self.current_field_size as usize..];
        visitor.visit_unit()
    }
}

struct SeqAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,

    count: u32,
}

impl<'a, 'de> serde::de::SeqAccess<'de> for SeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.count == 0 {
            self.de.dynamic_data_slice = None;

            return Ok(None);
        }
        self.count -= 1;

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.count as usize)
    }
}

struct MapAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,

    fields: std::slice::Iter<'de, Field>,
}

impl<'a, 'de> serde::de::MapAccess<'de> for MapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if let Some(field) = self.fields.next() {
            self.de.current_field_hash = field.hash;
            self.de.current_field_size = field.size;
            if self.de.current_field_size == 0 {
                self.de.current_field_size = self.de.data.read_u32::<LittleEndian>()?;
            }

            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}
