use std::io::Write;
use meta::metadata::{Compression, Entry, Header};
use meta::error::CatError;
use meta::utils::EvalContext;
use meta::utils::{wrap_context, write_string, write_u16, write_u32};

pub trait CatSerializable {
    fn serialize(&self, writer: &mut impl Write, context: EvalContext) -> meta::error::Result<()>;
}

impl CatSerializable for Compression {
    fn serialize(&self, writer: &mut impl Write, context: EvalContext) -> meta::error::Result<()> {
        wrap_context(
            match self {
                Compression::Gzip => writer.write(&[0xFEu8]),
                Compression::None => writer.write(&[0xFFu8]),
            },
            context,
            CatError::ErrorWritingMetadata,
        )?;

        Ok(())
    }
}

impl CatSerializable for Entry {
    fn serialize(&self, buffer: &mut impl Write, context: EvalContext) -> meta::error::Result<()> {
        match self {
            Entry::Directory { name, entries } => {
                wrap_context(
                    buffer.write(&[1]),
                    context.push("entry type".to_string()),
                    CatError::ErrorWritingMetadata,
                )?;
                wrap_context(
                    write_string(name, buffer),
                    context.push("file name".to_string()),
                    CatError::ErrorWritingMetadata,
                )?;
                wrap_context(
                    write_u16(
                        &wrap_context(
                            u16::try_from(entries.len()),
                            context.push("entry length".to_string()),
                            CatError::ErrorWritingMetadata,
                        )?,
                        buffer,
                    ),
                    context.push("entry length".to_string()),
                    CatError::ErrorWritingMetadata,
                )?;
                for x in entries {
                    x.serialize(buffer, context.push(name.to_string()))?
                }
            }
            Entry::File {
                name,
                offset,
                size,
                compression,
            } => {
                wrap_context(
                    buffer.write(&[0]),
                    context.push("entry type".to_string()),
                    CatError::ErrorWritingMetadata,
                )?;
                wrap_context(
                    write_string(name, buffer),
                    context.push("file name".to_string()),
                    CatError::ErrorWritingMetadata,
                )?;
                wrap_context(
                    write_u32(offset, buffer),
                    context.push("file offset".to_string()),
                    CatError::ErrorWritingMetadata,
                )?;
                wrap_context(
                    write_u32(size, buffer),
                    context.push("file size".to_string()),
                    CatError::ErrorWritingMetadata,
                )?;
                compression.serialize(
                    buffer,
                    context
                        .push(name.to_string())
                        .push("compression".to_string()),
                )?;
            }
        }

        Ok(())
    }
}


impl CatSerializable for Header {
    fn serialize(&self, writer: &mut impl Write, context: EvalContext) -> meta::error::Result<()> {
        wrap_context(
            writer.write(&[self.version]),
            context.push("version".to_string()),
            CatError::ErrorWritingMetadata,
        )?;
        wrap_context(
            write_u16(
                &wrap_context(
                    u16::try_from(self.entries.len()),
                    context.push("entries length".to_string()),
                    CatError::ErrorWritingMetadata,
                )?,
                writer,
            ),
            context.push("entries length".to_string()),
            CatError::ErrorWritingMetadata,
        )?;
        for entries in &self.entries {
            entries.serialize(writer, context.push("head".to_string()))?
        }
        Ok(())
    }
}