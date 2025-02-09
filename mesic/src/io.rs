use byteorder::{ByteOrder, LittleEndian};
use std::fs;
use std::io::Write;

pub fn write_as_bytes(floats: &Vec<f32>, filename: String) -> Result<(), std::io::Error> {
    let mut bytes: Vec<u8> = vec![0; floats.len() * 4];
    LittleEndian::write_f32_into(&floats.as_slice(), &mut bytes);

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(filename)?;

    let _ = file.write_all(&bytes);

    Ok(())
}
