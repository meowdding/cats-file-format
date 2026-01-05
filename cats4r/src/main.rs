mod deserializing;
mod error;
mod metadata;
mod packing;
mod serializing;
mod unpacking;
mod utils;

use crate::packing::pack;
use crate::unpacking::unpack;
use clap::{Arg, ArgMatches};
use clap::{ArgAction, Command};
use error::{ErrorType, Result};
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
            Arg::new("gzip")
                .short('n')
                .long("no-gzip")
                .action(ArgAction::SetFalse),
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
                    gzip: false,
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
                    gzip: matches.get_flag("gzip"),
                },
            )
        }
        _ => ErrorType::UnknownArg.into(),
    }
}

struct Context {
    verbose: bool,
    gzip: bool,
}
