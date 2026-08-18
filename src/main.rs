mod application;
mod audio;
mod clock;
mod configuration;
mod digits;
mod duration_parser;
mod interface;
mod stopwatch;
mod time_format;
mod timer;

use clap::{Parser, Subcommand};
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::execute;

use application::{Application, Tab, TimerScreen};
use audio::AudioPlayer;
use timer::Timer;

#[derive(Parser)]
#[command(about = "A clock for the terminal")]
struct Arguments {
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(long, global = true, help = "Hide the tab bar and help hint")]
    fullscreen: bool,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Count down a duration, then keep counting past zero")]
    Timer {
        #[arg(help = "Duration like 5, 90s, 1h30m or 5:00; omit to enter it interactively")]
        duration: Option<String>,
    },
    #[command(about = "Show the current time")]
    Time,
    #[command(about = "Count elapsed time")]
    Stopwatch,
}

fn main() -> std::process::ExitCode {
    // Names the PipeWire node (shown in mixers); must run before any thread exists.
    unsafe {
        std::env::set_var(
            "PIPEWIRE_ALSA",
            r#"{ application.name = "Clock" media.name = "Clock" node.name = "clock" node.description = "Clock" }"#,
        )
    };
    let arguments = Arguments::parse();
    let mut timer_screen = TimerScreen::Input {
        input: String::new(),
        error: None,
    };
    let tab = match arguments.command {
        None | Some(Command::Timer { duration: None }) => Tab::Timer,
        Some(Command::Timer {
            duration: Some(text),
        }) => match duration_parser::parse(&text) {
            Ok(duration) => {
                timer_screen = TimerScreen::Running {
                    timer: Timer::new(duration),
                    overtime_started: false,
                };
                Tab::Timer
            }
            Err(message) => {
                eprintln!("invalid duration: {message}");
                return std::process::ExitCode::FAILURE;
            }
        },
        Some(Command::Time) => Tab::Clock,
        Some(Command::Stopwatch) => Tab::Stopwatch,
    };

    let configuration = configuration::load();
    let audio_player = AudioPlayer::new(&configuration);
    let mut terminal = ratatui::init();
    let _ = execute!(std::io::stdout(), EnableMouseCapture);
    let restoring_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = execute!(std::io::stdout(), DisableMouseCapture);
        application::reset_terminal_background();
        restoring_hook(panic_info);
    }));
    let application = Application::new(
        tab,
        timer_screen,
        configuration,
        audio_player,
        arguments.fullscreen,
    );
    let result = application.run(&mut terminal);
    let _ = execute!(std::io::stdout(), DisableMouseCapture);
    ratatui::restore();
    application::reset_terminal_background();
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
