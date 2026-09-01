# clock

A clock for the terminal: countdown timer, current time, and stopwatch,
rendered as large half-block digits that scale to the window.

- The timer keeps counting into negatives past zero, flips the entire
  terminal background to a deep red, and plays a chime.
- Optional elevator music plays while the timer runs, from a file or a
  directory of tracks.
- Colors default to the terminal's own palette, so transparency is
  preserved. Backgrounds are set through OSC 11, which fills the whole
  window including padding; the escape is honored by most terminals
  (ghostty, kitty, foot, alacritty, wezterm, xterm).

## Build

```sh
mise run build
cp target/release/clock ~/bin/
```

## Usage

```sh
clock timer 5m        # start a five-minute countdown
clock timer           # enter a duration interactively
clock time            # show the current time
clock stopwatch       # count elapsed time
clock --fullscreen …  # hide the tab bar
```

Durations accept a bare number of minutes (`5`), unit suffixes (`90s`,
`5m`, `1h30m`), and colon forms (`5:00`, `1:00:00`).

## Keys

| Key | Action |
| --- | --- |
| `F1` `F2` `F3` | jump to a tab (clicking a tab works too) |
| `←` `→` / `h` `l` | cycle tabs |
| `f` | toggle fullscreen |
| `m` | music on / off, with a fade |
| `?` | help popup |
| `q` / `esc` | quit — on a running timer, back to duration entry |

Timer: `enter` starts, `space` pauses and resumes (the colons blink
while paused), `r` restarts. Stopwatch: `space` starts and pauses, `r`
resets.

## Configuration

`~/.config/clock/config.toml` (or the equivalent XDG config path) is
created with defaults on first run. All keys are optional.

```toml
# Colors are hex strings; unset means the terminal's own colors.
# background = "#2b3a55"
# foreground = "#f4ede4"
master_volume = 0.5                   # multiplies every sound, 1.0 = 100%

[timer]
overtime_background = "#7f1d1d"       # window color past zero
overtime_foreground = "#f4ede4"       # digit color past zero
chime_enabled = true
chime_volume = 1.0
# chime_path = "/path/to/sound.wav"   # unset: a built-in chime
music_enabled = false
# music_path = "/path/to/file-or-dir" # unset: ~/.config/clock/music
music_order = "random"                # or "sequential", for directories
music_volume = 0.7

[clock]
format = "24h"                        # or "12h"
# timezone = "America/New_York"       # unset: the system timezone
date = "%A, %B %-d, %Y"               # strftime pattern; "" hides it
# message = "text shown under the date"
```

Music plays while a timer runs, keeps playing through overtime, and
fades over a second when paused, muted, or stopped. A directory is
scanned for `wav`, `mp3`, `ogg`, `flac`, and `m4a` files; `random`
reshuffles on every pass through the playlist.

## Sounds

The chime is embedded in the binary and reproducible from source:

```sh
cargo run --example generate_chime            # rewrites assets/chime.wav
cargo run --example generate_elevator_music ~/.config/clock/music/elevator.wav
```

The second command synthesizes a seamless lounge loop to use as timer
music.
