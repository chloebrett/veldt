use byteorder::{ByteOrder, LittleEndian};
use std::fs;
use std::io::Write;

pub fn as_bytes(floats: &Vec<f32>) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![0; floats.len() * 4];
    LittleEndian::write_f32_into(&floats.as_slice(), &mut bytes);

    bytes
}

pub fn as_floats(bytes: &Vec<u8>) -> Vec<f32> {
    let mut floats: Vec<f32> = vec![0.0; bytes.len() / 4];
    LittleEndian::read_f32_into(&bytes.as_slice(), &mut floats);

    floats
}

pub fn write_as_bytes(floats: &Vec<f32>, filename: String) -> Result<(), std::io::Error> {
    let bytes = as_bytes(floats);

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(filename)?;

    let _ = file.write_all(&bytes);

    Ok(())
}
