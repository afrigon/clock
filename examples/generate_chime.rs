use std::f32::consts::TAU;

const SAMPLE_RATE: u32 = 44_100;
const DURATION_SECONDS: f32 = 1.4;

fn main() {
    let total_samples = (SAMPLE_RATE as f32 * DURATION_SECONDS) as usize;
    let mut samples = vec![0.0f32; total_samples];
    strike(&mut samples, 0.0, 660.0);
    strike(&mut samples, 0.25, 880.0);
    normalize(&mut samples, 0.8);

    let mut bytes = Vec::with_capacity(44 + samples.len() * 2);
    let data_size = (samples.len() * 2) as u32;
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&((sample * i16::MAX as f32) as i16).to_le_bytes());
    }

    std::fs::create_dir_all("assets").unwrap();
    std::fs::write("assets/chime.wav", bytes).unwrap();
}

fn strike(samples: &mut [f32], start_seconds: f32, frequency: f32) {
    let start = (start_seconds * SAMPLE_RATE as f32) as usize;
    for (index, sample) in samples.iter_mut().enumerate().skip(start) {
        let time = (index - start) as f32 / SAMPLE_RATE as f32;
        let envelope = (-4.0 * time).exp();
        *sample += envelope
            * ((TAU * frequency * time).sin()
                + 0.4 * (-6.0 * time).exp() * (TAU * frequency * 2.0 * time).sin()
                + 0.15 * (-9.0 * time).exp() * (TAU * frequency * 3.0 * time).sin());
    }
}

fn normalize(samples: &mut [f32], peak_target: f32) {
    let peak = samples.iter().fold(0.0f32, |peak, sample| peak.max(sample.abs()));
    if peak > 0.0 {
        for sample in samples.iter_mut() {
            *sample *= peak_target / peak;
        }
    }
}
