use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use flate2::read::GzDecoder;
use meta::Context;
use crate::deserializing::CatDeserializable;
use meta::error::CatError;
use meta::metadata::{Compression, Entry, Header, MAGIC_NUMBER};
use meta::utils::{validate_name, EvalContext};

pub fn unpack(directory: &Path, source: &Path, context: &Context) -> meta::error::Result<()> {
    if !source.is_file() {
        eprintln!("Can't read input as it's not a file!");
        return Err(CatError::InvalidInput(source.display().to_string()).into());
    }

    let mut file = File::open(source).map_err(|err| {
        CatError::FailedToOpenInput {
            path: source.display().to_string(),
            error: err.to_string(),
        }
            .into()
    })?;

    let mut file_magic_number = [0u8; 4];
    file.read_exact(&mut file_magic_number).map_err(|err| {
        CatError::ErrorReadingFile {
            path: source.display().to_string(),
            error: err.to_string(),
        }
            .into()
    })?;
    if !file_magic_number.eq(&MAGIC_NUMBER) {
        return Err(CatError::InvalidFileType.into());
    }

    let header = Header::deserialize(&mut file, EvalContext::new("header".to_string()))?;

    let mut content = Vec::<u8>::new();
    match file.read_to_end(&mut content) {
        Err(err) => {
            return CatError::ErrorReadingFile {
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
) -> meta::error::Result<()> {
    match entry {
        Entry::Directory { name, entries } => {
            let mut new_path = PathBuf::from(path);
            new_path.push(validate_name(name.clone(), &eval_context)?);
            if context.verbose {
                println!("Unpacking {}", new_path.display());
            }
            for entry in entries {
                unpack_entry(data, new_path.as_path(), entry, context, &eval_context.push(name.clone()))?
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
                        return CatError::InvalidEntryData(eval_context.clone()).into();
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
                return CatError::UnableToCreateDirectory(parent.display().to_string()).into();
            }
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
                .unwrap()
                .write_all(actual_content)
                .map_err(|err| {
                    CatError::ErrorWritingFile {
                        path: path.display().to_string(),
                        error: err.to_string(),
                    }
                        .into()
                })
        }
    }
}
