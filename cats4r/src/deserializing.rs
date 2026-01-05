use std::io::Read;
use crate::utils::EvalContext;
use crate::utils::{read_string, read_u16, read_u32, read_u8, wrap_context};
use crate::error::ErrorType;
use crate::metadata::{Compression, Entry, Header};

pub trait CatDeserializable {
    fn deserialize(reader: &mut impl Read, context: EvalContext) -> crate::error::Result<Self>
    where
        Self: Sized;
}

impl CatDeserializable for Compression {
    fn deserialize(reader: &mut impl Read, context: EvalContext) -> crate::error::Result<Compression> {
        let value = wrap_context(read_u8(reader), context, ErrorType::ErrorReadingMetadata)?;
        match value {
            0xFE => Ok(Compression::Gzip),
            0xFF => Ok(Compression::None),
            _ => panic!("Invalid compression! {}", value),
        }
    }
}

impl CatDeserializable for Entry {
    fn deserialize(reader: &mut impl Read, context: EvalContext) -> crate::error::Result<Entry> {
        let data = wrap_context(
            read_u8(reader),
            context.push("entry type".to_string()),
            ErrorType::ErrorReadingMetadata,
        )?;
        match data {
            0 => {
                let name = wrap_context(
                    read_string(reader),
                    context.push("file name".to_string()),
                    ErrorType::ErrorReadingMetadata,
                )?;
                let offset = wrap_context(
                    read_u32(reader),
                    context.push("offset".to_string()),
                    ErrorType::ErrorReadingMetadata,
                )?;
                let size = wrap_context(
                    read_u32(reader),
                    context.push("size".to_string()),
                    ErrorType::ErrorReadingMetadata,
                )?;
                let compression = Compression::deserialize(
                    reader,
                    context
                        .push(name.to_string())
                        .push("compression".to_string()),
                )?;

                Ok(Entry::File {
                    name,
                    offset,
                    size,
                    compression,
                })
            }
            1 => {
                let name = wrap_context(
                    read_string(reader),
                    context.push("directory name".to_string()),
                    ErrorType::ErrorReadingMetadata,
                )?;
                let amount = wrap_context(
                    read_u16(reader),
                    context.push("directory name".to_string()),
                    ErrorType::ErrorReadingMetadata,
                )?;
                let mut entries = Vec::<Entry>::new();
                for _ in 0..amount {
                    entries.push(Entry::deserialize(reader, context.push(name.to_string()))?)
                }

                Ok(Entry::Directory { name, entries })
            }
            _ => ErrorType::InvalidEntryType(context, data).into(),
        }
    }
}

impl CatDeserializable for Header {
    fn deserialize(reader: &mut impl Read, context: EvalContext) -> crate::error::Result<Header> {
        let version = wrap_context(
            read_u8(reader),
            context.push("version".to_string()),
            ErrorType::ErrorReadingMetadata,
        )?;
        let amount = wrap_context(
            read_u16(reader),
            context.push("entries length".to_string()),
            ErrorType::ErrorReadingMetadata,
        )?;
        let mut entries = Vec::<Entry>::new();
        for i in 0..amount {
            entries.push(Entry::deserialize(reader, context.push(i.to_string()))?.clone())
        }

        Ok(Header { version, entries })
    }
}
