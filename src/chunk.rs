use core::fmt;
use std::io::{BufReader, Read};

use anyhow::{ensure, Context, Error, Result};

use crate::chunk_type::ChunkType;

#[derive(Debug)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    chunk_data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, chunk_data: Vec<u8>) -> Self {
        let length = chunk_data.len() as u32;

        let mut bytes_to_calc: Vec<u8> = Vec::new();
        bytes_to_calc.extend(&chunk_type.bytes());
        bytes_to_calc.extend(&chunk_data);
        let crc = Self::calculate_crc(&bytes_to_calc);

        Self {
            length,
            chunk_type,
            chunk_data,
            crc,
        }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.chunk_data
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data_as_string(&self) -> Result<String> {
        String::from_utf8(self.chunk_data.clone()).context("Unable to get message from data")
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let chunk_data: Vec<u8> = self
            .length
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type.bytes().iter())
            .chain(self.chunk_data.iter())
            .chain(self.crc.to_be_bytes().iter())
            .copied()
            .collect();

        chunk_data
    }

    fn calculate_crc(bytes: &[u8]) -> u32 {
        let crc: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        crc.checksum(bytes)
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Chunk {{",)?;
        writeln!(f, "  Length: {}", self.length())?;
        writeln!(f, "  Type: {}", self.chunk_type())?;
        writeln!(f, "  Data: {} bytes", self.data().len())?;
        writeln!(f, "  Crc: {}", self.crc())?;
        writeln!(f, "}}",)?;
        Ok(())
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(data);

        let mut length_buffer: [u8; 4] = [0; 4];
        reader
            .read_exact(&mut length_buffer)
            .context("Unable to read length bytes")?;
        let length = u32::from_be_bytes(length_buffer);

        let mut chunk_type_buffer: [u8; 4] = [0; 4];
        reader
            .read_exact(&mut chunk_type_buffer)
            .context("Unable to read chunk type bytes")?;

        let chunk_type = ChunkType::try_from(chunk_type_buffer)?;

        let mut data_buffer: Vec<u8> = vec![0u8; length as usize];
        reader
            .read_exact(&mut data_buffer)
            .context("Unable to read data bytes")?;

        let mut crc_buffer = [0u8; 4];
        reader
            .read_exact(&mut crc_buffer)
            .context("Unable to read crc bytes")?;

        let crc = u32::from_be_bytes(crc_buffer);

        let mut bytes_to_calc: Vec<u8> = Vec::new();
        bytes_to_calc.extend(&chunk_type_buffer);
        bytes_to_calc.extend(&data_buffer);
        let calculated_crc = Self::calculate_crc(&bytes_to_calc);

        ensure!(
            crc == calculated_crc,
            "Invalid CRC, received: {}, expected: {}",
            crc,
            calculated_crc
        );

        Ok(Self {
            length,
            chunk_type,
            chunk_data: data_buffer,
            crc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
