use clap::{Args, Parser, Subcommand};

#[derive(Args, Debug)]
pub struct EncodeArgs {
    /// File path of the image
    pub file_path: String,

    /// Type of the chunk containing the hidden message. A 4-letter (a-zA-Z) string where the cases
    /// of each letter should be: [lowercase], [lowercase], [UPPERCASE], [lowercase] respectively. Ex.: ruSt
    pub chunk_type: String,

    /// The message
    pub message: String,

    /// Optional output file, if not specified the original image is overwritten
    pub output_file: Option<String>,
}

#[derive(Args, Debug)]
pub struct DecodeArgs {
    /// File path of the image
    pub file_path: String,

    /// Type of the chunk containing the hidden message
    pub chunk_type: String,
}

#[derive(Args, Debug)]
pub struct PrintArgs {
    /// File path of the image
    pub file_path: String,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// File path of the image
    pub file_path: String,

    /// Type of the chunk containing the hidden message
    pub chunk_type: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Hides a message in an image by storing it in a non-critical chunk.
    Encode(EncodeArgs),

    /// Prints hidden messages in chunks of a specific chunk type
    Decode(DecodeArgs),

    /// Prints all chunks of a PNG image
    Print(PrintArgs),

    /// Removes all chunks of a specific chunk type. This will overwrite the file.
    Remove(RemoveArgs),
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
