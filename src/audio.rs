use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use rand::seq::SliceRandom;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

use crate::configuration::{Configuration, MusicOrder};

static EMBEDDED_CHIME: &[u8] = include_bytes!("../assets/chime.wav");

const MUSIC_EXTENSIONS: [&str; 5] = ["wav", "mp3", "ogg", "flac", "m4a"];

pub struct AudioPlayer {
    _device_sink: MixerDeviceSink,
    effects: Player,
    music: Player,
}

impl AudioPlayer {
    pub fn new(configuration: &Configuration) -> Option<Self> {
        let mut device_sink = DeviceSinkBuilder::open_default_sink().ok()?;
        device_sink.log_on_drop(false);
        let effects = Player::connect_new(device_sink.mixer());
        let music = Player::connect_new(device_sink.mixer());
        effects.set_volume(configuration.master_volume * configuration.timer.chime_volume);
        music.set_volume(configuration.master_volume * configuration.timer.music_volume);
        Some(Self {
            _device_sink: device_sink,
            effects,
            music,
        })
    }

    pub fn play_chime(&self, chime_path: Option<&Path>) {
        if let Some(decoder) = chime_path
            .and_then(|path| File::open(path).ok())
            .and_then(|file| Decoder::try_from(file).ok())
        {
            self.effects.append(decoder);
            return;
        }
        if let Ok(decoder) = Decoder::new(Cursor::new(EMBEDDED_CHIME)) {
            self.effects.append(decoder);
        }
    }

    pub fn music_idle(&self) -> bool {
        self.music.empty()
    }

    pub fn queue_music(&self, files: &[PathBuf]) -> usize {
        let mut queued = 0;
        for file in files {
            if let Some(decoder) = File::open(file)
                .ok()
                .and_then(|file| Decoder::try_from(file).ok())
            {
                self.music.append(decoder);
                queued += 1;
            }
        }
        if queued > 0 {
            self.music.play();
        }
        queued
    }

    pub fn set_music_volume(&self, volume: f32) {
        self.music.set_volume(volume);
    }

    pub fn pause_music(&self) {
        self.music.pause();
    }

    pub fn resume_music(&self) {
        self.music.play();
    }

    pub fn stop_music(&self) {
        self.music.clear();
    }
}

pub fn music_playlist(path: &Path, order: MusicOrder) -> Vec<PathBuf> {
    let mut files = if path.is_dir() {
        std::fs::read_dir(path)
            .map(|entries| {
                entries
                    .flatten()
                    .map(|entry| entry.path())
                    .filter(|path| {
                        path.extension()
                            .and_then(|extension| extension.to_str())
                            .is_some_and(|extension| {
                                MUSIC_EXTENSIONS.contains(&extension.to_lowercase().as_str())
                            })
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![path.to_path_buf()]
    };
    match order {
        MusicOrder::Sequential => files.sort(),
        MusicOrder::Random => files.shuffle(&mut rand::rng()),
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playlist_filters_and_orders_directory_entries() {
        let directory = std::env::temp_dir().join(format!("clock-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        for name in ["b.wav", "a.mp3", "notes.txt", "c.OGG"] {
            std::fs::write(directory.join(name), b"").unwrap();
        }
        let sequential = music_playlist(&directory, MusicOrder::Sequential);
        let names: Vec<_> = sequential
            .iter()
            .map(|path| path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, ["a.mp3", "b.wav", "c.OGG"]);
        assert_eq!(music_playlist(&directory, MusicOrder::Random).len(), 3);
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn playlist_for_a_single_file_is_that_file() {
        let file = Path::new("/some/song.mp3");
        assert_eq!(
            music_playlist(file, MusicOrder::Sequential),
            vec![file.to_path_buf()]
        );
    }
}
