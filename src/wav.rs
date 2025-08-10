use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

use ndarray::Array1;

pub fn save_array1_f32_as_wav<P: AsRef<Path>>(
    data: &Array1<f32>,
    out_path: P,
    sample_rate: Option<u32>,
) -> Result<(), io::Error> {
    let mut file = File::create(out_path)?;
    let sample_rate = sample_rate.unwrap_or(22000);

    let num_samples = data.len() as u32;
    let num_channels = 1u16;
    let bits_per_sample = 32u16;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_size = num_samples * (bits_per_sample as u32 / 8);
    let file_size = 36 + data_size;

    file.write_all(b"RIFF")?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;

    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&3u16.to_le_bytes())?;
    file.write_all(&num_channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;

    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;

    for &sample in data {
        file.write_all(&sample.to_le_bytes())?;
    }

    Ok(())
}
