use clap::{Arg};
use clap::{ArgAction, Command};
use hex::ToHex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

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
        _ => exit(2),
    }
}

struct Context {
    verbose: bool,
}

const MAGIC_NUMBER: [u8; 4] = [0x43, 0x41, 0x54, 0x53];

fn unpack(directory: &Path, source: &Path, context: &Context) {
    if !source.is_file() {
        eprintln!("Can't read from {} as it's not a file!", source.display());
        exit(-1);
    }

    let mut file = match File::open(source) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open file {} ({})", source.display(), err);
            exit(-2)
        }
    };

    let mut file_magic_number = [0u8; 4];
    file.read_exact(&mut file_magic_number).expect("");
    if !file_magic_number.eq(&MAGIC_NUMBER) {
        eprintln!(
            "Unsupported input file type! {:?} {}",
            file_magic_number,
            source.display()
        );
        exit(-3);
    }

    let header = Header::deserialize(&mut file);

    let mut content = Vec::<u8>::new();
    file.read_to_end(&mut content).expect("TODO: panic message");
    for entry in header.entries {
        unpack_entry(&content, &directory, &entry, &context)
    }
}

fn validate_name(name: &String) -> Option<&String> {
    if name.chars().all(|c| c.is_ascii_graphic() && c != '/' && c != '\\') && name != ".." {
        return Some(name);
    }
    None
}

fn unpack_entry(data: &Vec<u8>, path: &Path, entry: &Entry, context: &Context) {
    match entry {
        Entry::Directory { name, entries } => {
            let mut new_path = PathBuf::from(path);
            new_path.push(match validate_name(&name) {
                Some(name) => name,
                None => {
                    eprintln!("Invalid entry name {}!", name);
                    exit(-5)
                }
            });
            if context.verbose {
                println!("Unpacking {}", new_path.display());
            }
            for entry in entries {
                unpack_entry(data, new_path.as_path(), entry, context)
            }
        }
        Entry::File {
            name,
            offset,
            size,
            compression,
        } => {
            let mut new_path = PathBuf::from(path);
            new_path.push(match validate_name(&name) {
                Some(name) => name,
                None => {
                    eprintln!("Invalid entry name {}!", name);
                    exit(-5)
                }
            });

            let offset = *offset as usize;
            let end = offset + *size as usize;
            let content = &data[offset..end];

            let actual_content = match compression {
                Compression::Gzip => panic!("Not yet implemented"),
                Compression::None => content,
            };

            if context.verbose {
                println!("Unpacking {}", new_path.display());
            }

            let path = new_path.as_path();
            fs::create_dir_all(path.parent().unwrap())
                .expect("Failed to create parent directories");
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
                .unwrap()
                .write_all(actual_content)
                .expect("TODO: panic message");
        }
    }
}

fn pack(directory: &Path, target: &Path, context: &Context) {
    let mut serialized = HashMap::<String, EntryData>::new();
    let mut data = Vec::<u8>::new();
    let entry = create_entry(directory, &mut serialized, &mut data, context);

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
    file.write_all(&MAGIC_NUMBER).expect("TODO: panic message"); // magic number
    header.serialize(&mut file);
    file.write_all(data.as_slice())
        .expect("TODO: panic message");
    file.flush().unwrap()
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
) -> Entry {
    if path.is_dir() {
        if context.verbose {
            println!("Serializing directory {}", path.display())
        }
        let mut entries = Vec::<Entry>::new();
        entries.extend(
            fs::read_dir(path)
                .unwrap()
                .into_iter()
                .filter_map(Result::ok)
                .map(|x| create_entry(x.path().as_path(), serialized, vec, context)),
        );
        let name = path.file_name().unwrap().to_str().unwrap().to_string();

        return Entry::Directory {
            name: match validate_name(&name) {
                Some(name) => name.to_string(),
                None => {
                    eprintln!("Invalid entry name {}!", name);
                    exit(-5)
                }
            },
            entries,
        };
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
            let offset = u32::try_from(vec.len()).expect("TODO: panic message");
            let size = u32::try_from(content.len()).expect("TODO: panic message");

            serialized.insert(hash, EntryData { size, offset });
            vec.extend(content);

            &EntryData { size, offset }
        };
        let name = path.file_name().unwrap().to_str().unwrap().to_string();

        return Entry::File {
            name: match validate_name(&name) {
                Some(name) => name.to_string(),
                None => {
                    eprintln!("Invalid entry name {}!", name);
                    exit(-5)
                }
            },
            offset: data.offset,
            size: data.size,
            compression: Compression::None,
        };
    }

    panic!("Failed to create entry {}", path.display());
}

trait CatSerializable {
    fn serialize(&self, writer: &mut impl Write);
}
trait CatDeserializable {
    fn deserialize(reader: &mut impl Read) -> Self;
}

#[derive(Debug)]
enum Compression {
    Gzip = 0xFE,
    None = 0xFF,
}

impl CatSerializable for Compression {
    fn serialize(&self, writer: &mut impl Write) {
        match self {
            Compression::Gzip => writer.write(&[0xFEu8]).expect("TODO: panic message"),
            Compression::None => writer.write(&[0xFFu8]).expect("TODO: panic message"),
        };
    }
}

impl CatDeserializable for Compression {
    fn deserialize(reader: &mut impl Read) -> Compression {
        let value = read_u8(reader);
        match value {
            0xFE => Compression::Gzip,
            0xFF => Compression::None,
            _ => panic!("Invalid compression! {}", value),
        }
    }
}

#[derive(Debug)]
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

fn write_u32(value: &u32, buffer: &mut impl Write) {
    buffer
        .write_all(&u32::to_be_bytes(*value))
        .expect("TODO: panic message");
}

fn write_u16(value: &u16, buffer: &mut impl Write) {
    buffer
        .write_all(&u16::to_be_bytes(*value))
        .expect("TODO: panic message");
}

fn write_string(string: &String, buffer: &mut impl Write) {
    let bytes = string.as_bytes();
    let length = bytes.len();
    buffer
        .write_all(&[length as u8])
        .expect("TODO: panic message");
    buffer.write_all(bytes).expect("TODO: panic message");
}

fn read_string(buffer: &mut impl Read) -> String {
    let mut size = [0u8; 1];
    buffer.read_exact(&mut size).expect("TODO: panic message");
    let mut string = &mut Vec::with_capacity(size[0] as usize);
    buffer
        .take(size[0] as u64)
        .read_to_end(&mut string)
        .expect("TODO: panic message");
    String::from_utf8_lossy(string).to_string()
}

fn read_u8(buffer: &mut impl Read) -> u8 {
    let mut number = [0u8; 1];
    buffer.read_exact(&mut number).expect("TODO: panic message");
    number[0]
}

fn read_u32(buffer: &mut impl Read) -> u32 {
    let mut number = [0u8; 4];
    buffer.read_exact(&mut number).expect("TODO: panic message");
    u32::from_be_bytes(number)
}

fn read_u16(buffer: &mut impl Read) -> u16 {
    let mut number = [0u8; 2];
    buffer.read_exact(&mut number).expect("TODO: panic message");
    u16::from_be_bytes(number)
}

impl CatSerializable for Entry {
    fn serialize(&self, buffer: &mut impl Write) {
        match self {
            Entry::Directory { name, entries } => {
                buffer.write(&[1]).expect("TODO: panic message");
                write_string(name, buffer);
                write_u16(
                    &u16::try_from(entries.len()).expect("TODO: panic message"),
                    buffer,
                );
                for x in entries {
                    x.serialize(buffer)
                }
            }
            Entry::File {
                name,
                offset,
                size,
                compression,
            } => {
                buffer.write(&[0]).expect("TODO: panic message");
                write_string(name, buffer);
                write_u32(offset, buffer);
                write_u32(size, buffer);
                compression.serialize(buffer);
            }
        };
    }
}
impl CatDeserializable for Entry {
    fn deserialize(reader: &mut impl Read) -> Entry {
        match read_u8(reader) {
            0 => {
                let name = read_string(reader);
                let offset = read_u32(reader);
                let size = read_u32(reader);
                let compression = Compression::deserialize(reader);

                Entry::File {
                    name,
                    offset,
                    size,
                    compression,
                }
            }
            1 => {
                let name = read_string(reader);
                let amount = read_u16(reader);
                let mut entries = Vec::<Entry>::new();
                for _ in 0..amount {
                    entries.push(Entry::deserialize(reader))
                }

                Entry::Directory { name, entries }
            }
            _ => {
                panic!("Invalid archive!")
            }
        }
    }
}

#[derive(Debug)]
struct Header {
    version: u8,
    entries: Vec<Entry>,
}

impl CatSerializable for Header {
    fn serialize(&self, writer: &mut impl Write) {
        writer.write(&[self.version]).expect("TODO: panic message");
        write_u16(
            &u16::try_from(self.entries.len()).expect("TODO: panic message"),
            writer,
        );
        for entries in &self.entries {
            entries.serialize(writer)
        }
    }
}
impl CatDeserializable for Header {
    fn deserialize(reader: &mut impl Read) -> Header {
        let version = read_u8(reader);
        let amount = read_u16(reader);
        let mut entries = Vec::<Entry>::new();
        for _ in 0..amount {
            entries.push(Entry::deserialize(reader))
        }

        Header { version, entries }
    }
}
