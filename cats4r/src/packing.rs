use std::collections::HashMap;
use std::fs;
use std::fs::{DirEntry, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use flate2::read::GzEncoder;
use crate::Context;
use crate::error::ErrorType;
use crate::metadata::{Compression, Entry, Header, MAGIC_NUMBER};
use crate::utils::{validate_name, wrap_context, EvalContext};
use hex::ToHex;
use sha2::{Digest, Sha256};
use crate::serializing::CatSerializable;

pub fn pack(directory: &Path, target: &Path, context: &Context) -> crate::error::Result<()> {
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
) -> crate::error::Result<Entry> {
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
                &eval_context.push(x.path().as_path().display().to_string()),
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
            let mut buff = Vec::<u8>::new();
            let content = if context.gzip {
                let mut meow = GzEncoder::new(&content[..], flate2::Compression::best());
                wrap_context(
                    meow.read_to_end(&mut buff),
                    eval_context.push("gzip".to_string()),
                    ErrorType::FailedToCompressData,
                )?;
                buff
            } else {
                content
            };
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
