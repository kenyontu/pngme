use anyhow::{Context, Result};
use std::{fs, io::Write, path::Path, str::FromStr};

use clap::Parser;
use cli::{Cli, Commands, DecodeArgs, EncodeArgs, PrintArgs, RemoveArgs};

use crate::{chunk::Chunk, chunk_type::ChunkType, png::Png};

mod chunk;
mod chunk_type;
mod cli;
mod png;

/// Hides a message in an image by storing it in a non-critical chunk
fn encode(args: EncodeArgs) -> Result<()> {
    let chunk_type = ChunkType::from_str(&args.chunk_type)?;
    chunk_type.is_valid_for_message()?;

    let path = Path::new(&args.file_path);
    let mut png = Png::from_file(path).context("Unable to load image file")?;

    let data: Vec<u8> = args.message.bytes().collect();
    let message_chunk = Chunk::new(chunk_type, data);

    png.append_chunk(message_chunk);

    let destination = args.output_file.unwrap_or(args.file_path);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&destination)
        .context("Unable to open image file to write")?;

    file.write_all(&png.as_bytes())
        .context("Error writing image file")?;

    println!("Message successfuly encoded");

    Ok(())
}

/// Prints hidden messages in chunks of a specific chunk type
fn decode(args: DecodeArgs) -> Result<()> {
    let path = Path::new(&args.file_path);
    let mut png = Png::from_file(path).context("Unable to load image file")?;

    let mut messages: Vec<String> = Vec::new();
    let mut chunks_with_problem = 0;

    loop {
        let chunk = match png.remove_first_chunk(&args.chunk_type) {
            Some(chunk) => chunk,
            _ => break,
        };

        if let Ok(message) = chunk.data_as_string() {
            messages.push(message);
        } else {
            chunks_with_problem += 1;
        }
    }

    if messages.len() > 0 {
        println!("Messages:");
        println!("{}", messages.join("\n"));
    }

    if chunks_with_problem > 0 {
        println!(
            "\nUnable to read data from {} chunk(s)",
            chunks_with_problem
        );
    }

    if messages.len() == 0 {
        println!("No chunks with chunk type \"{}\" found", args.chunk_type);
    }

    Ok(())
}

/// Prints all chunks of an image
fn print(args: PrintArgs) -> Result<()> {
    let path = Path::new(&args.file_path);
    let png = Png::from_file(path).context("Unable to load image file")?;

    println!("{}", png);

    Ok(())
}

/// Removes all chunks of a specific chunk type. This will overwrite the file.
fn remove(args: RemoveArgs) -> Result<()> {
    let path = Path::new(&args.file_path);
    let mut png = Png::from_file(path).context("Unable to load image file")?;

    let mut removed_chunk_count = 0;

    loop {
        match png.remove_first_chunk(&args.chunk_type) {
            Some(_) => removed_chunk_count += 1,
            _ => break,
        };
    }

    if removed_chunk_count > 0 {
        let mut file = fs::OpenOptions::new()
            .write(true)
            // Truncate empties the file after opening it, this is necessary since we want to
            // replace its contents
            .truncate(true)
            .open(&args.file_path)
            .context("Unable to open image file to write")?;

        file.write_all(&png.as_bytes())
            .context("Error writing image file")?;

        println!("Number of chunks removed: {}", removed_chunk_count);
    } else {
        println!("No chunk with chunk type \"{}\" found", args.chunk_type);
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Encode(args) => encode(args),
        Commands::Decode(args) => decode(args),
        Commands::Print(args) => print(args),
        Commands::Remove(args) => remove(args),
    };

    if let Err(e) = result {
        println!("Error: {}", e);
    }
}
