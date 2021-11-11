use std::{
    error::Error,
    fs::{self, read_to_string},
    path::PathBuf,
};

use rs2ts::{Converter, TsType};

// (Full example with detailed comments in examples/01d_quick_example.rs)
//
// This example demonstrates clap's full 'custom derive' style of creating arguments which is the
// simplest method of use, but sacrifices some flexibility.
use clap::Clap;
use syn::Type;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
struct Opts {
    /// Some input. Because this isn't an Option<T> it's required to be used
    input: Vec<PathBuf>,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,

    #[clap(short, long)]
    outfile: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    println!("Using input file: {:?}", opts.input);

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    match opts.verbose {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        _ => println!("Don't be crazy"),
    }

    let mut conv = Converter::new();
    conv.add_overwriter(Box::new(|t| {
        match t {
            Type::Path(type_path) => {
                let last = type_path.path.segments.last().unwrap();
                let field_type = last.ident.to_string();

                match field_type.as_str() {
                    "UserId" | "GuildId" | "ChannelId" | "RoleId" | "ApplicationId" | "EmojiId"
                    | "MessageId" | "StickerPackId" | "AttachmentId" | "WebhookId"
                    | "StickerId" | "GenericId" => Some(TsType::String),
                    _ => None,
                }
            }
            _ => {
                // println!("{:?}", &field.ty);
                None
            }
        }
    }));

    let mut result = String::new();
    for file_path in opts.input {
        let contents = read_to_string(file_path)?;
        let ts = conv.parse_from_file(&contents);
        ts.iter()
            .map(|e| result.push_str(&format!("{}\n\n", e)))
            .count();
    }

    if let Some(outfile) = opts.outfile {
        fs::write(outfile, &result).unwrap();
    } else {
        println!("{}", result);
    }

    Ok(())
}
