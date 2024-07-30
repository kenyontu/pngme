use std::{fmt::Display, str::FromStr};

use anyhow::{ bail, ensure, Context, Error, Result};

mod error {
    pub fn invalid_length(str_chunk_type: &str) -> String {
        format!(
            "The chunk type \"{}\" is invalid, it should have 4 characters",
            str_chunk_type
        )
    }

    pub fn invalid_character(str_chunk_type: &str, invalid_char: &char, pos: &usize) -> String {
        format!("The chunk type \"{}\" contains the invalid character '{}' at position {}, it should only contain characters that match the [a-zA-Z] pattern", str_chunk_type, invalid_char, pos)
    }
}

#[derive(Debug, PartialEq)]
pub struct ChunkType {
    bytes: [u8; 4],
}

impl ChunkType {
    /// Checks if the chunk type is appropriate for storing hidden messages
    pub fn is_valid_for_message(&self) -> Result<()> {
        // Should be a valid chunk type
        self.is_valid()?;

        // Should be non-critical
        ensure!(
            !self.is_critical(), 
            "The 1st letter in the chunk type \"{}\" is uppercase, which marks the chunk as critical. Hidden messages should be hidden in non-critical chunks, so change it to be lowercase.", self.to_string()
        );

        // Should be private
        ensure!(
            !self.is_public(),
            "The 2nd letter in the chunk type \"{}\" is uppercase, which marks the chunk as public. Hidden messages should be hidden in private chunks, so change it to be lowercase.", self.to_string()
        );

        // Should be safe to copy
        ensure!(
            self.is_safe_to_copy(),
            "The 4th letter in the chunk type \"{}\" is uppercase, which marks the chunk as unsafe to copy. Hidden messages should be hidden in safe to copy chunks, so change it to be lowercase.", self.to_string()
        );

        Ok(())
    }

    /// Checks if the length of this chunk type is valid
    pub fn is_length_valid(&self) -> bool {
        self.bytes.len() == 4
    }

    /// Checks if the chunk type is valid according to the PNG spec
    pub fn is_valid(&self) -> Result<()> {
        if let Some((c, i)) = Self::validate_chars(&self.bytes) {
            bail!(error::invalid_character(&self.to_string(), &c, &i));
        }

        ensure!(
            self.is_length_valid(),
            error::invalid_length(&self.to_string())
        );

        ensure!(
            self.is_reserved_bit_valid(),
            "The 3rd letter in the chunk type \"{}\" is lowercase, the PNG spec requires this letter to be uppercase.", self.to_string()
        );

        Ok(())
    }

    pub fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    /// A chunk type is critical if:
    ///
    /// - It's 1st character is uppercase; or
    /// - The 5th bit of the first byte is 1
    pub fn is_critical(&self) -> bool {
        (self.bytes[0] & (1 << 5)) == 0
    }

    /// A chunk type is public if:
    ///
    /// - It's 2nd character is uppercase; or
    /// - The 5th bit of the second byte is 1
    pub fn is_public(&self) -> bool {
        (self.bytes[1] & (1 << 5)) == 0
    }

    /// The reserved bit of a chunk type is valid if:
    ///
    /// - It's 3rd character is uppercase; or
    /// - The 5th bit of the third byte is 1
    pub fn is_reserved_bit_valid(&self) -> bool {
        (self.bytes[2] & (1 << 5)) == 0
    }

    /// A chunk is safe to copy if:
    ///
    /// - It's 4th character is lowercase; or
    /// - The 5th bit of the fourth byte is 0
    pub fn is_safe_to_copy(&self) -> bool {
        (self.bytes[3] & (1 << 5)) > 0
    }

    /// Checks if a slice of bytes contain non-ascii letters.
    ///
    /// If there's an invalid character, returns a tuple with the invalid character and its
    /// position (zero-based).
    pub fn validate_chars(bytes: &[u8]) -> Option<(char, usize)> {
        for (i, byte) in bytes.iter().enumerate() {
            let is_valid = byte.is_ascii_lowercase() || byte.is_ascii_uppercase();
            if !is_valid {
                return Some((*byte as char, i));
            }
        }

        None
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    fn try_from(bytes: [u8; 4]) -> Result<Self, Self::Error> {
        let chunk_type = Self { bytes };

        chunk_type.is_valid()?;

        Ok(chunk_type)
    }
}

impl FromStr for ChunkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();

        // From the tests, this should return an error if the bytes are not a valid lowercase or
        // uppercase ascii character
        if let Some((c, i)) = Self::validate_chars(bytes) {
            bail!(error::invalid_character(s, &c, &i));
        }

        ensure!(bytes.len() == 4, error::invalid_length(s));

        let chunk_type = Self {
            bytes: bytes.try_into().context("Failed to convert bytes")?,
        };

        Ok(chunk_type)
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.bytes.map(|b| b as char).iter().collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid().is_ok());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(chunk.is_valid().is_err());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
