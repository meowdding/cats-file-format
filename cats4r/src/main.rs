mod error;
mod utils;

use crate::utils::EvalContext;
use clap::{Arg, ArgMatches};
use clap::{ArgAction, Command};
use error::{ErrorType, Result};
use flate2::read::GzDecoder;
use hex::ToHex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::{DirEntry, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use utils::validate_name;

fn main() {
    let matches = Command::new("Cats Archiver")
        .author("Mona, mona@mona.cat")
        .version("1.0.0")
        .subcommand(
            Command::new("unpack")
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("archive_name")
                        .required(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("destination")
                        .required(false)
                        .action(ArgAction::Set),
                ),
        )
        .subcommand_negates_reqs(true)
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("archive_name")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(Arg::new("input_dir").action(ArgAction::Set))
        .get_matches();

    match handle(matches) {
        Ok(_) => exit(0),
        Err(err) => {
            eprintln!("An error occurred!\n{err}");
            exit(err.exit_code())
        }
    }
}

fn handle(matches: ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("unpack", command)) => {
            let archive_name = Path::new(
                command
                    .get_one::<String>("archive_name")
                    .expect("Expected archive name to be present!"),
            );

            let mut buffer = PathBuf::from(archive_name);
            let output = match command.get_one::<String>("destination") {
                Some(file) => Path::new(file),
                None => {
                    buffer.set_extension("");
                    buffer.as_path()
                }
            };

            unpack(
                output,
                archive_name,
                &Context {
                    verbose: command.get_flag("verbose"),
                },
            )
        }

        None => {
            let archive_name = Path::new(
                matches
                    .get_one::<String>("archive_name")
                    .expect("Expected archive name to be present!"),
            );

            let input = match matches.get_one::<String>("input_dir") {
                Some(file) => Path::new(file),
                None => Path::new("."),
            };

            pack(
                input,
                archive_name,
                &Context {
                    verbose: matches.get_flag("verbose"),
                },
            )
        }
        _ => ErrorType::UnknownArg.into(),
    }
}

struct Context {
    verbose: bool,
}

const MAGIC_NUMBER: [u8; 4] = [0x43, 0x41, 0x54, 0x53];

fn unpack(directory: &Path, source: &Path, context: &Context) -> Result<()> {
    if !source.is_file() {
        eprintln!("Can't read input as it's not a file!");
        return Err(ErrorType::InvalidInput(source.display().to_string()).into());
    }

    let mut file = File::open(source).map_err(|err| {
        ErrorType::FailedToOpenInput {
            path: source.display().to_string(),
            error: err.to_string(),
        }
        .into()
    })?;

    let mut file_magic_number = [0u8; 4];
    file.read_exact(&mut file_magic_number).map_err(|err| {
        ErrorType::ErrorReadingFile {
            path: source.display().to_string(),
            error: err.to_string(),
        }
        .into()
    })?;
    if !file_magic_number.eq(&MAGIC_NUMBER) {
        return Err(ErrorType::InvalidFileType.into());
    }

    let header = Header::deserialize(&mut file, EvalContext::new("header".to_string()))?;

    let mut content = Vec::<u8>::new();
    match file.read_to_end(&mut content) {
        Err(err) => {
            return ErrorType::ErrorReadingFile {
                path: source.display().to_string(),
                error: err.to_string(),
            }
            .into();
        }
        _ => {}
    }
    for entry in &header.entries {
        unpack_entry(
            &content,
            &directory,
            &entry,
            &context,
            &EvalContext::new("unpacking".to_string()),
        )?
    }

    Ok(())
}

fn unpack_entry(
    data: &Vec<u8>,
    path: &Path,
    entry: &Entry,
    context: &Context,
    eval_context: &EvalContext,
) -> Result<()> {
    match entry {
        Entry::Directory { name, entries } => {
            let mut new_path = PathBuf::from(path);
            new_path.push(validate_name(name.clone(), &eval_context)?);
            if context.verbose {
                println!("Unpacking {}", new_path.display());
            }
            for entry in entries {
                unpack_entry(data, new_path.as_path(), entry, context, &eval_context)?
            }

            Ok(())
        }
        Entry::File {
            name,
            offset,
            size,
            compression,
        } => {
            let mut new_path = PathBuf::from(path);
            new_path.push(validate_name(name.clone(), &eval_context)?);

            let offset = *offset as usize;
            let end = offset + *size as usize;
            let content = &data[offset..end];

            let mut vec = Vec::<u8>::new();
            let actual_content = match compression {
                Compression::Gzip => {
                    if GzDecoder::new(content).read_to_end(&mut vec).is_err() {
                        return ErrorType::InvalidEntryData(eval_context.clone()).into();
                    }
                    &vec
                }
                Compression::None => content,
            };

            if context.verbose {
                println!("Unpacking {}", new_path.display());
            }

            let path = new_path.as_path();
            let parent = path.parent().unwrap();
            if fs::create_dir_all(parent).is_err() {
                return ErrorType::UnableToCreateDirectory(parent.display().to_string()).into();
            }
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
                .unwrap()
                .write_all(actual_content)
                .map_err(|err| {
                    ErrorType::ErrorWritingFile {
                        path: path.display().to_string(),
                        error: err.to_string(),
                    }
                    .into()
                })
        }
    }
}

fn pack(directory: &Path, target: &Path, context: &Context) -> Result<()> {
    let mut serialized = HashMap::<String, EntryData>::new();
    let mut data = Vec::<u8>::new();
    let entry = create_entry(
        directory,
        &mut serialized,
        &mut data,
        context,
        &EvalContext::new("Archiving".to_string()),
    )?;

    let header = match entry {
        Entry::File { .. } => panic!("Root is file and not directory!"),
        Entry::Directory { entries, .. } => Header {
            version: 1,
            entries,
        },
    };

    let mut file = match OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(target)
    {
        Ok(file) => file,
        Err(err) => panic!("Failed to create file {} ({})", target.display(), err),
    };
    file.write_all(&MAGIC_NUMBER).map_err(|x| {
        ErrorType::ErrorWritingFile {
            path: target.display().to_string(),
            error: x.to_string(),
        }
        .new()
    })?;
    header.serialize(&mut file, EvalContext::new("pack".to_string()))?;
    file.write_all(data.as_slice()).map_err(|x| {
        ErrorType::ErrorWritingFile {
            path: target.display().to_string(),
            error: x.to_string(),
        }
        .new()
    })?;
    file.flush().map_err(|x| {
        ErrorType::ErrorWritingFile {
            path: target.display().to_string(),
            error: x.to_string(),
        }
        .new()
    })?;

    Ok(())
}

struct EntryData {
    size: u32,
    offset: u32,
}

fn create_entry(
    path: &Path,
    serialized: &mut HashMap<String, EntryData>,
    vec: &mut Vec<u8>,
    context: &Context,
    eval_context: &EvalContext,
) -> Result<Entry> {
    if path.is_dir() {
        if context.verbose {
            println!("Serializing directory {}", path.display())
        }
        let mut entries = Vec::<Entry>::new();
        for x in fs::read_dir(path)
            .unwrap()
            .into_iter()
            .filter_map(std::result::Result::ok)
            .collect::<Vec<DirEntry>>()
        {
            entries.push(create_entry(
                x.path().as_path(),
                serialized,
                vec,
                context,
                eval_context,
            )?)
        }
        let name = path.file_name().unwrap().to_str().unwrap().to_string();

        return Ok(Entry::Directory {
            name: validate_name(name.clone(), eval_context)?,
            entries,
        });
    }
    if path.is_file() {
        if context.verbose {
            println!("Serializing file {}", path.display())
        }
        let content = fs::read(path).unwrap();
        let hash = Sha256::digest(&content).encode_hex::<String>();

        let data = if serialized.contains_key(&hash) {
            serialized.get(&hash).unwrap()
        } else {
            let offset = u32::try_from(vec.len()).expect("Failed to convert usize to u32");
            let size = u32::try_from(content.len()).expect("Failed to convert usize to u32");

            serialized.insert(hash, EntryData { size, offset });
            vec.extend(content);

            &EntryData { size, offset }
        };
        let name = path.file_name().unwrap().to_str().unwrap().to_string();

        return Ok(Entry::File {
            name: validate_name(name.clone(), eval_context)?,
            offset: data.offset,
            size: data.size,
            compression: Compression::None,
        });
    }

    panic!("Failed to create entry {}", path.display());
}

trait CatSerializable {
    fn serialize(&self, writer: &mut impl Write, context: EvalContext) -> Result<()>;
}
trait CatDeserializable {
    fn deserialize(reader: &mut impl Read, context: EvalContext) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
enum Compression {
    Gzip = 0xFE,
    None = 0xFF,
}

impl CatSerializable for Compression {
    fn serialize(&self, writer: &mut impl Write, context: EvalContext) -> Result<()> {
        wrap_context(
            match self {
                Compression::Gzip => writer.write(&[0xFEu8]),
                Compression::None => writer.write(&[0xFFu8]),
            },
            context,
            ErrorType::ErrorWritingMetadata,
        )?;

        Ok(())
    }
}

impl CatDeserializable for Compression {
    fn deserialize(reader: &mut impl Read, context: EvalContext) -> Result<Compression> {
        let value = wrap_context(read_u8(reader), context, ErrorType::ErrorReadingMetadata)?;
        match value {
            0xFE => Ok(Compression::Gzip),
            0xFF => Ok(Compression::None),
            _ => panic!("Invalid compression! {}", value),
        }
    }
}

#[derive(Debug, Clone)]
enum Entry {
    Directory {
        name: String,
        entries: Vec<Entry>,
    },
    File {
        name: String,
        offset: u32,
        size: u32,
        compression: Compression,
    },
}

fn write_u32(value: &u32, buffer: &mut impl Write) -> std::result::Result<(), std::io::Error> {
    buffer.write_all(&u32::to_be_bytes(*value))
}

fn write_u16(value: &u16, buffer: &mut impl Write) -> std::result::Result<(), std::io::Error> {
    buffer.write_all(&u16::to_be_bytes(*value))
}

fn write_string(
    string: &String,
    buffer: &mut impl Write,
) -> std::result::Result<(), std::io::Error> {
    let bytes = string.as_bytes();
    let length = bytes.len();
    buffer.write_all(&[length as u8])?;
    buffer.write_all(bytes)
}

fn read_string(buffer: &mut impl Read) -> std::result::Result<String, std::io::Error> {
    let mut size = [0u8; 1];
    buffer.read_exact(&mut size)?;
    let mut string = &mut Vec::with_capacity(size[0] as usize);
    buffer.take(size[0] as u64).read_to_end(&mut string)?;
    Ok(String::from_utf8_lossy(string).to_string())
}

fn read_u8(buffer: &mut impl Read) -> std::result::Result<u8, std::io::Error> {
    let mut number = [0u8; 1];
    buffer.read_exact(&mut number)?;
    Ok(number[0])
}

fn read_u32(buffer: &mut impl Read) -> std::result::Result<u32, std::io::Error> {
    let mut number = [0u8; 4];
    buffer.read_exact(&mut number)?;
    Ok(u32::from_be_bytes(number))
}

fn read_u16(buffer: &mut impl Read) -> std::result::Result<u16, std::io::Error> {
    let mut number = [0u8; 2];
    buffer.read_exact(&mut number)?;
    Ok(u16::from_be_bytes(number))
}

impl CatSerializable for Entry {
    fn serialize(&self, buffer: &mut impl Write, context: EvalContext) -> Result<()> {
        match self {
            Entry::Directory { name, entries } => {
                wrap_context(
                    buffer.write(&[1]),
                    context.push("entry type".to_string()),
                    ErrorType::ErrorWritingMetadata,
                )?;
                wrap_context(
                    write_string(name, buffer),
                    context.push("file name".to_string()),
                    ErrorType::ErrorWritingMetadata,
                )?;
                wrap_context(
                    write_u16(
                        &wrap_context(
                            u16::try_from(entries.len()),
                            context.push("entry length".to_string()),
                            ErrorType::ErrorWritingMetadata,
                        )?,
                        buffer,
                    ),
                    context.push("entry length".to_string()),
                    ErrorType::ErrorWritingMetadata,
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
                    ErrorType::ErrorWritingMetadata,
                )?;
                wrap_context(
                    write_string(name, buffer),
                    context.push("file name".to_string()),
                    ErrorType::ErrorWritingMetadata,
                )?;
                wrap_context(
                    write_u32(offset, buffer),
                    context.push("file offset".to_string()),
                    ErrorType::ErrorWritingMetadata,
                )?;
                wrap_context(
                    write_u32(size, buffer),
                    context.push("file size".to_string()),
                    ErrorType::ErrorWritingMetadata,
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
impl CatDeserializable for Entry {
    fn deserialize(reader: &mut impl Read, context: EvalContext) -> Result<Entry> {
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

#[derive(Debug)]
struct Header {
    version: u8,
    entries: Vec<Entry>,
}

fn wrap_context<T, E>(
    result: std::result::Result<T, E>,
    context: EvalContext,
    converter: fn(EvalContext, String) -> ErrorType,
) -> Result<T>
where
    E: Error,
{
    result.map_err(|err| converter(context.clone(), err.to_string()).into())
}

impl CatSerializable for Header {
    fn serialize(&self, writer: &mut impl Write, context: EvalContext) -> Result<()> {
        wrap_context(
            writer.write(&[self.version]),
            context.push("version".to_string()),
            ErrorType::ErrorWritingMetadata,
        )?;
        wrap_context(
            write_u16(
                &wrap_context(
                    u16::try_from(self.entries.len()),
                    context.push("entries length".to_string()),
                    ErrorType::ErrorWritingMetadata,
                )?,
                writer,
            ),
            context.push("entries length".to_string()),
            ErrorType::ErrorWritingMetadata,
        )?;
        for entries in &self.entries {
            entries.serialize(writer, context.push("head".to_string()))?
        }
        Ok(())
    }
}
impl CatDeserializable for Header {
    fn deserialize(reader: &mut impl Read, context: EvalContext) -> Result<Header> {
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
