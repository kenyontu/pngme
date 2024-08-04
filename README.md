# PNGme

A PNGme implementation following the instructions from the [PNGme Book](https://jrdngr.github.io/pngme_book/).

This is a practice project where you learn how to work with bytes by building a CLI application that encodes and decodes text messages in PNG images

# How are messges hidden?

PNG files consist of a specific header followed by a series of chunks. Each chunk has a type used to identify it's contents, some contain the image data while others contain additional information like last modified data, color space information, etc.

PNGs may also contain chunks with custom types that can hold any kind of data. Since these chunks are not in the PNG spec, they are normaly ignored by image decoders and require specialized software to read them. Messages are stored in these chunks.

# Commands

## Encode a message

To hide a message in a PNG file, use the sub-command `encode`:

```bash
pngme encode <file> <chunk_type> <message> [output_file]

# Examples:

# Adds a hidden message to cat.png in a chunk of type "ruSt"
pngme encode cat.png ruSt "Hi!"

# Creates a copy of cat.png in cat2.png containing the hidden message in a chunk of type "ruSt". The original image is untouched
pngme encode cat.png ruSt "Hi" cat2.png
```

Parameters:

- **file**: The png image file path
- **chunk_type**: A valid custom chunk type. Ex: `ruSt`, `aaAa` and `foOo`. See below how to define valid custom chunk types for messages
- **message**: The message
- **output_file**: Optional. If specified, a new image will be created with the contents of the original image plus the hidden message. Otherwise the original image will be overwritten.

### Custom Chunk Types

Chunk types are composed of 4 letters, in the ranges a-z and A-Z of the english alphabet, eg. `abCd`. Depending on whether a letter is uppercase or not, it can have a different meaning.

To store messages, the cases for each letter should be `LLUL`, where `L` stands for lowercase and `U` for uppercase. Check the reasons below:

| Position | Case      | Reason                                                                                                   |
| -------- | --------- | -------------------------------------------------------------------------------------------------------- |
| 1st      | lowercase | Marks the chunk as non-critical, meaning that it doesn't contain image data                              |
| 2nd      | lowercase | Marks the chunk as private, to indicate that it is a custom chunk not in the PNG spec                    |
| 3rd      | uppercase | This letter should always be uppercase                                                                   |
| 4th      | lowercase | Tells image editors that this chunk can be safely copied over to the new version of the image when saved |

For more details check the [PNG Spec](http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html), chapter 3.3 Chunk naming conventions.

## Decode messages

To decode messages use the `decode` sub-command:

```bash
pngme decode <file> <chunk_type>

# Example:

# Decodes messages in cat.png with the chunk ruSt
pngme decode cat.png ruSt
```

- **file**: The png image file path
- **chunk_type**: The type of chunk with hidden messages to decode

## Remove chunks by type

To remove chunks by type, use the `remove` sub-command:

```bash
pngme remove <file> <chunk_type>

# Example:

# Removes chunks matching the ruSt type from cat.png
pngme remove cat.png ruSt
```

- **file**: The png image file path
- **chunk_type**: The type of chunk to remove. All chunks matching this type will be removed

## Print private chunks

To check an image for chunks possibly containing messages, use the `print` sub-command:

```bash
pngme print <file>

# Example:

# Prints chunks from cat.png
pngme print cat.png
```

- **file**: The png image file path
