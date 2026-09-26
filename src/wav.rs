use std::io::{self, Write};
use crate::buffer::AudioBuffer;
use crate::SfxrError;

/// Serializes an AudioBuffer to raw WAV (RIFF/WAVE 16-bit Mono PCM) byte vector.
pub fn to_wav_bytes(buffer: &AudioBuffer) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(44 + buffer.len() * 2);
    write_wav(&mut bytes, buffer).expect("Writing to Vec should never fail");
    bytes
}

/// Writes an AudioBuffer to any `std::io::Write` stream in standard 16-bit Mono WAV format.
pub fn write_wav<W: Write>(writer: &mut W, buffer: &AudioBuffer) -> io::Result<()> {
    let num_samples = buffer.len() as u32;
    let sample_rate = buffer.sample_rate;
    let bits_per_sample = 16u16;
    let num_channels = 1u16;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_size = num_samples * block_align as u32;
    let riff_chunk_size = 36 + data_size;

    // RIFF chunk descriptor
    writer.write_all(b"RIFF")?;
    writer.write_all(&riff_chunk_size.to_le_bytes())?;
    writer.write_all(b"WAVE")?;

    // "fmt " subchunk
    writer.write_all(b"fmt ")?;
    writer.write_all(&16u32.to_le_bytes())?; // Subchunk1Size = 16 for PCM
    writer.write_all(&1u16.to_le_bytes())?;  // AudioFormat = 1 (PCM)
    writer.write_all(&num_channels.to_le_bytes())?;
    writer.write_all(&sample_rate.to_le_bytes())?;
    writer.write_all(&byte_rate.to_le_bytes())?;
    writer.write_all(&block_align.to_le_bytes())?;
    writer.write_all(&bits_per_sample.to_le_bytes())?;

    // "data" subchunk
    writer.write_all(b"data")?;
    writer.write_all(&data_size.to_le_bytes())?;

    // Write 16-bit PCM samples
    let pcm_samples = buffer.to_i16();
    for sample in pcm_samples {
        writer.write_all(&sample.to_le_bytes())?;
    }

    writer.flush()?;
    Ok(())
}

/// Validates and parses the header of a 16-bit PCM WAV byte slice.
/// Returns `(sample_rate, num_channels, num_samples)`.
pub fn parse_wav_info(bytes: &[u8]) -> Result<(u32, u16, usize), SfxrError> {
    if bytes.len() < 44 {
        return Err(SfxrError::InvalidWavHeader("WAV data too short (< 44 bytes)".into()));
    }

    if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(SfxrError::InvalidWavHeader("Missing RIFF/WAVE header".into()));
    }

    if &bytes[12..16] != b"fmt " {
        return Err(SfxrError::InvalidWavHeader("Missing fmt subchunk".into()));
    }

    let channels = u16::from_le_bytes([bytes[22], bytes[23]]);
    let sample_rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
    let bits_per_sample = u16::from_le_bytes([bytes[34], bytes[35]]);

    if bits_per_sample != 16 {
        return Err(SfxrError::InvalidWavHeader(format!(
            "Unsupported bits per sample: expected 16, got {bits_per_sample}"
        )));
    }

    if &bytes[36..40] != b"data" {
        return Err(SfxrError::InvalidWavHeader("Missing data subchunk".into()));
    }

    let data_len = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]) as usize;
    let num_samples = data_len / (channels as usize * (bits_per_sample as usize / 8));

    Ok((sample_rate, channels, num_samples))
}
