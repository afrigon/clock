use std::io::Write;
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use ratatui::style::Color;
use ratatui::DefaultTerminal;

use crate::audio::{self, AudioPlayer};
use crate::configuration::Configuration;
use crate::duration_parser;
use crate::interface;
use crate::stopwatch::Stopwatch;
use crate::timer::Timer;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Timer,
    Clock,
    Stopwatch,
}

impl Tab {
    fn previous(self) -> Self {
        match self {
            Self::Timer => Self::Stopwatch,
            Self::Clock => Self::Timer,
            Self::Stopwatch => Self::Clock,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Timer => Self::Clock,
            Self::Clock => Self::Stopwatch,
            Self::Stopwatch => Self::Timer,
        }
    }
}

pub enum TimerScreen {
    Input {
        input: String,
        error: Option<String>,
    },
    Running {
        timer: Timer,
        overtime_started: bool,
    },
}

impl TimerScreen {
    fn blank_input() -> Self {
        Self::Input {
            input: String::new(),
            error: None,
        }
    }
}

pub struct Application {
    pub tab: Tab,
    pub timer_screen: TimerScreen,
    pub stopwatch: Stopwatch,
    pub configuration: Configuration,
    pub help_visible: bool,
    pub fullscreen: bool,
    audio_player: Option<AudioPlayer>,
    music_unavailable: bool,
    music_enabled: bool,
    music_level: f32,
    overtime_background_applied: bool,
}

const MUSIC_FADE_SECONDS: f32 = 1.0;

impl Application {
    pub fn new(
        tab: Tab,
        timer_screen: TimerScreen,
        configuration: Configuration,
        audio_player: Option<AudioPlayer>,
        fullscreen: bool,
    ) -> Self {
        let music_enabled = configuration.timer.music_enabled;
        Self {
            tab,
            timer_screen,
            stopwatch: Stopwatch::new(),
            configuration,
            help_visible: false,
            fullscreen,
            audio_player,
            music_unavailable: false,
            music_enabled,
            music_level: 1.0,
            overtime_background_applied: false,
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        set_terminal_background(self.configuration.background);
        let mut previous_tick = Instant::now();
        loop {
            terminal.draw(|frame| interface::draw(frame, &self))?;
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key)
                        if key.kind == KeyEventKind::Press
                            && self.handle_key(key.code, key.modifiers) =>
                    {
                        return Ok(());
                    }
                    Event::Mouse(mouse)
                        if mouse.kind == MouseEventKind::Down(MouseButton::Left)
                            && !self.fullscreen
                            && !self.help_visible
                            && mouse.row == 0 =>
                    {
                        if let Some(tab) = interface::tab_at(mouse.column) {
                            self.tab = tab;
                        }
                    }
                    _ => {}
                }
            }
            self.handle_zero_crossing();
            self.synchronize_background();
            let now = Instant::now();
            self.advance_music_fade(now.duration_since(previous_tick).as_secs_f32());
            previous_tick = now;
            self.maintain_music();
        }
    }

    fn advance_music_fade(&mut self, delta_seconds: f32) {
        let Some(audio_player) = &self.audio_player else {
            return;
        };
        let on_running_screen = matches!(self.timer_screen, TimerScreen::Running { .. });
        let counting = matches!(
            &self.timer_screen,
            TimerScreen::Running { timer, .. } if !timer.is_paused()
        );
        let audible = self.music_enabled && counting;
        let previous_level = self.music_level;
        let step = delta_seconds / MUSIC_FADE_SECONDS;
        if audible {
            self.music_level = (self.music_level + step).min(1.0);
            if previous_level == 0.0 {
                audio_player.resume_music();
            }
        } else {
            self.music_level = (self.music_level - step).max(0.0);
            if self.music_level == 0.0 && previous_level > 0.0 {
                if on_running_screen {
                    audio_player.pause_music();
                } else {
                    audio_player.stop_music();
                }
            }
        }
        if self.music_level != previous_level {
            audio_player.set_music_volume(
                self.configuration.master_volume
                    * self.configuration.timer.music_volume
                    * self.music_level,
            );
        }
    }

    fn synchronize_background(&mut self) {
        let overtime_visible = self.tab == Tab::Timer
            && matches!(
                self.timer_screen,
                TimerScreen::Running {
                    overtime_started: true,
                    ..
                }
            );
        if overtime_visible == self.overtime_background_applied {
            return;
        }
        self.overtime_background_applied = overtime_visible;
        if overtime_visible {
            set_terminal_background(self.configuration.timer.overtime_background);
        } else {
            apply_normal_background(&self.configuration);
        }
    }

    fn maintain_music(&mut self) {
        let counting = matches!(
            &self.timer_screen,
            TimerScreen::Running { timer, .. } if !timer.is_paused()
        );
        if self.music_unavailable || !self.music_enabled || !counting {
            return;
        }
        let Some(path) = &self.configuration.timer.music_path else {
            return;
        };
        let Some(audio_player) = &self.audio_player else {
            return;
        };
        if !audio_player.music_idle() {
            return;
        }
        let playlist = audio::music_playlist(path, self.configuration.timer.music_order);
        if audio_player.queue_music(&playlist) == 0 {
            self.music_unavailable = true;
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            return true;
        }
        if self.help_visible {
            if matches!(code, KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc) {
                self.help_visible = false;
            }
            return false;
        }
        let typing = self.tab == Tab::Timer && matches!(self.timer_screen, TimerScreen::Input { .. });
        match code {
            KeyCode::Char('?') => self.help_visible = true,
            KeyCode::Char('f') => self.fullscreen = !self.fullscreen,
            KeyCode::F(1) => self.tab = Tab::Timer,
            KeyCode::F(2) => self.tab = Tab::Clock,
            KeyCode::F(3) => self.tab = Tab::Stopwatch,
            KeyCode::Left => self.tab = self.tab.previous(),
            KeyCode::Right => self.tab = self.tab.next(),
            KeyCode::Char('h') if !typing => self.tab = self.tab.previous(),
            KeyCode::Char('l') if !typing => self.tab = self.tab.next(),
            KeyCode::Char('m') if !typing => self.music_enabled = !self.music_enabled,
            _ => {
                return match self.tab {
                    Tab::Timer => self.handle_timer_key(code),
                    Tab::Clock => matches!(code, KeyCode::Esc | KeyCode::Char('q')),
                    Tab::Stopwatch => self.handle_stopwatch_key(code),
                };
            }
        }
        false
    }

    fn handle_timer_key(&mut self, code: KeyCode) -> bool {
        match &mut self.timer_screen {
            TimerScreen::Input { input, error } => match code {
                KeyCode::Esc | KeyCode::Char('q') => return true,
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => match duration_parser::parse(input) {
                    Ok(duration) => {
                        self.timer_screen = TimerScreen::Running {
                            timer: Timer::new(duration),
                            overtime_started: false,
                        };
                    }
                    Err(message) => *error = Some(message),
                },
                KeyCode::Char(character) if "0123456789hms:".contains(character) => {
                    input.push(character);
                    *error = None;
                }
                _ => {}
            },
            TimerScreen::Running {
                timer,
                overtime_started,
            } => match code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    if self.music_level == 0.0
                        && let Some(audio_player) = &self.audio_player
                    {
                        audio_player.stop_music();
                    }
                    self.timer_screen = TimerScreen::blank_input();
                }
                KeyCode::Char('r') => {
                    timer.restart();
                    *overtime_started = false;
                }
                KeyCode::Char(' ') => timer.toggle_paused(),
                _ => {}
            },
        }
        false
    }

    fn handle_stopwatch_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => return true,
            KeyCode::Char(' ') => self.stopwatch.toggle(),
            KeyCode::Char('r') => self.stopwatch.reset(),
            _ => {}
        }
        false
    }

    fn handle_zero_crossing(&mut self) {
        let TimerScreen::Running {
            timer,
            overtime_started,
        } = &mut self.timer_screen
        else {
            return;
        };
        if *overtime_started || timer.remaining_milliseconds() > 0 {
            return;
        }
        *overtime_started = true;
        if !self.configuration.timer.chime_enabled {
            return;
        }
        if let Some(audio_player) = &self.audio_player {
            audio_player.play_chime(self.configuration.timer.chime_path.as_deref());
        }
    }
}

fn set_terminal_background(color: Color) {
    if let Color::Rgb(red, green, blue) = color {
        print!("\x1b]11;rgb:{red:02x}/{green:02x}/{blue:02x}\x07");
        let _ = std::io::stdout().flush();
    }
}

fn apply_normal_background(configuration: &Configuration) {
    if matches!(configuration.background, Color::Rgb(..)) {
        set_terminal_background(configuration.background);
    } else {
        reset_terminal_background();
    }
}

pub fn reset_terminal_background() {
    print!("\x1b]111\x07");
    let _ = std::io::stdout().flush();
}
