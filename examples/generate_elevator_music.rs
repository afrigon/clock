use std::f32::consts::TAU;

const SAMPLE_RATE: u32 = 44_100;
const BEATS_PER_MINUTE: f32 = 104.0;
const BEATS: usize = 32;

const CHORDS: [[i32; 4]; 4] = [
    [50, 53, 57, 60],
    [53, 55, 59, 62],
    [52, 55, 59, 60],
    [55, 57, 60, 64],
];

const BASS: [[i32; 4]; 4] = [
    [38, 45, 41, 45],
    [43, 50, 47, 41],
    [36, 43, 40, 43],
    [45, 52, 48, 40],
];

const MELODY: [(f32, i32, f32); 14] = [
    (4.5, 76, 0.5),
    (5.0, 79, 0.5),
    (5.5, 81, 1.5),
    (12.5, 79, 0.5),
    (13.0, 76, 0.5),
    (13.5, 74, 0.5),
    (14.0, 72, 2.0),
    (20.5, 81, 0.5),
    (21.0, 79, 0.5),
    (21.5, 76, 0.5),
    (22.0, 74, 1.5),
    (28.0, 79, 0.75),
    (28.75, 76, 0.75),
    (29.5, 72, 2.0),
];

fn main() {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "elevator.wav".to_string());
    let seconds_per_beat = 60.0 / BEATS_PER_MINUTE;
    let total_samples = (BEATS as f32 * seconds_per_beat * SAMPLE_RATE as f32) as usize;
    let mut samples = vec![0.0f32; total_samples];

    for bar in 0..BEATS / 4 {
        let chord = CHORDS[bar % 4];
        let bar_start = bar as f32 * 4.0;
        for offset in [0.0, 2.5] {
            for note in chord {
                add_tone(
                    &mut samples,
                    (bar_start + offset) * seconds_per_beat,
                    1.4 * seconds_per_beat,
                    frequency(note),
                    0.16,
                    false,
                );
            }
        }
        for (beat, note) in BASS[bar % 4].iter().enumerate() {
            add_tone(
                &mut samples,
                (bar_start + beat as f32) * seconds_per_beat,
                0.95 * seconds_per_beat,
                frequency(*note),
                0.30,
                false,
            );
        }
        for beat in [0.0, 2.0] {
            add_kick(&mut samples, (bar_start + beat) * seconds_per_beat);
        }
        for beat in [1.0, 3.0] {
            add_noise(
                &mut samples,
                (bar_start + beat) * seconds_per_beat,
                0.09,
                0.08,
                false,
            );
        }
        for eighth in 0..8 {
            let beat = eighth as f32 * 0.5;
            let accent = eighth % 2 == 0;
            add_noise(
                &mut samples,
                (bar_start + beat) * seconds_per_beat,
                0.03,
                if accent { 0.10 } else { 0.05 },
                true,
            );
        }
    }

    for (beat, note, duration_beats) in MELODY {
        add_tone(
            &mut samples,
            beat * seconds_per_beat,
            duration_beats * seconds_per_beat,
            frequency(note),
            0.22,
            true,
        );
    }

    normalize(&mut samples, 0.85);
    write_wav(&output, &samples);
    println!("wrote {output}");
}

fn frequency(midi_note: i32) -> f32 {
    440.0 * ((midi_note - 69) as f32 / 12.0).exp2()
}

fn add_tone(
    samples: &mut [f32],
    start_seconds: f32,
    duration_seconds: f32,
    frequency: f32,
    amplitude: f32,
    vibrato: bool,
) {
    let start = (start_seconds * SAMPLE_RATE as f32) as usize;
    let length = (duration_seconds * 1.5 * SAMPLE_RATE as f32) as usize;
    let mut phase = 0.0f32;
    for offset in 0..length {
        let time = offset as f32 / SAMPLE_RATE as f32;
        let wobble = if vibrato {
            1.0 + 0.006 * (TAU * 5.0 * time).sin()
        } else {
            1.0
        };
        phase += TAU * frequency * wobble / SAMPLE_RATE as f32;
        let attack = (time / 0.012).min(1.0);
        let envelope = attack * (-3.0 * time / duration_seconds).exp();
        let value = phase.sin() + 0.35 * (2.0 * phase).sin() + 0.08 * (3.0 * phase).sin();
        samples[(start + offset) % samples.len()] += amplitude * envelope * value;
    }
}

fn add_kick(samples: &mut [f32], start_seconds: f32) {
    let start = (start_seconds * SAMPLE_RATE as f32) as usize;
    let length = (0.22 * SAMPLE_RATE as f32) as usize;
    let mut phase = 0.0f32;
    for offset in 0..length {
        let time = offset as f32 / SAMPLE_RATE as f32;
        let sweep = 85.0 * (-9.0 * time).exp() + 44.0;
        phase += TAU * sweep / SAMPLE_RATE as f32;
        let envelope = (-16.0 * time).exp();
        samples[(start + offset) % samples.len()] += 0.42 * envelope * phase.sin();
    }
}

fn add_noise(
    samples: &mut [f32],
    start_seconds: f32,
    decay_seconds: f32,
    amplitude: f32,
    bright: bool,
) {
    let start = (start_seconds * SAMPLE_RATE as f32) as usize;
    let length = (decay_seconds * 6.0 * SAMPLE_RATE as f32) as usize;
    let mut state = 0x2545_f491u32.wrapping_add(start as u32);
    let mut previous = 0.0f32;
    for offset in 0..length {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let white = state as f32 / u32::MAX as f32 * 2.0 - 1.0;
        let shaped = if bright {
            white - previous
        } else {
            (white + previous) * 0.5
        };
        previous = white;
        let time = offset as f32 / SAMPLE_RATE as f32;
        let envelope = (-time / decay_seconds).exp();
        samples[(start + offset) % samples.len()] += amplitude * envelope * shaped;
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

fn write_wav(path: &str, samples: &[f32]) {
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
    std::fs::write(path, bytes).unwrap();
}
