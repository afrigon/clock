# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
cargo test                     # unit tests (pure logic: parsing, formatting, scaling, config)
cargo test zero_crossing       # single test by name substring
cargo clippy --all-targets     # must stay at zero warnings (covers examples and tests)
cargo run -- timer 3s          # fastest full-path manual check (chime, red flip, negatives)
cargo build --release && cp target/release/clock ~/bin/clock   # install; rm first if "Text file busy"
cargo run --example generate_chime                             # rewrites assets/chime.wav
cargo run --example generate_elevator_music <output.wav>       # synthesizes timer music
```

The TUI cannot be driven interactively from a sandboxed session. To verify
rendering, run it in a pty and replay the escape stream:

```sh
(sleep 3; printf 'q'; sleep 0.4; printf 'q') | \
  script -qec "stty cols 100 rows 30; ./target/debug/clock timer 2s" /tmp/capture.txt
```

Parse `/tmp/capture.txt` by following cursor-position escapes into a grid to
reconstruct frames, and grep it for OSC sequences (`]11;`, `]111`) to assert
background changes. Note `q`/`esc` on a running timer returns to the duration
input screen — quitting from there takes a second `q`. To test with a scratch
config instead of the real one, set `XDG_CONFIG_HOME`.

## Conventions

- Zero code comments except 1-2 lines on genuinely non-obvious constraints
  (there are only a couple in the whole tree; match that bar).
- Full-word identifiers, no abbreviations (`configuration`, not `config`).
- Config values never brick startup: invalid entries `eprintln!` a warning and
  fall back to defaults (see `configuration.rs` — every field goes through a
  `*_or_default`/`validated_*` function).
- Durable text only in committed files: no "currently/new/recently" phrasing.

## Architecture

Single-binary ratatui application. `application.rs` owns all state and runs the
event loop; `interface.rs` draws; everything else is a leaf module.

**Event loop = tick loop.** `Application::run` polls crossterm events with a
50 ms timeout, so every iteration doubles as a tick. After input handling, three
per-tick reconciliation passes run, each idempotent and edge-triggered:

- `handle_zero_crossing` — fires the chime once when the timer first reaches
  zero (`overtime_started` flag re-armed by restart).
- `synchronize_background` — compares "should the window be red" (timer tab
  visible AND overtime) against what was last applied, and emits OSC only on
  change.
- `advance_music_fade` + `maintain_music` — ramps `music_level` (0..1) toward
  its target using real elapsed time (1 s full fade), applies
  master × music × level to the player, pauses/clears at zero; refills the
  playlist queue whenever it runs dry while the timer is actively counting
  (self-healing: covers CLI starts, Enter starts, and restarts with no
  special-casing).

**Backgrounds never touch ratatui cells.** Cells stay `Color::Reset`
throughout; window backgrounds (configured color, overtime red) are set with
OSC 11 and reset with OSC 111 so terminal padding is filled and transparency
survives. Every exit path must reset: normal exit and the wrapped panic hook in
`main.rs` both do. Do not paint cell backgrounds — it breaks transparency and
leaves the window-padding border, which is why this design exists.

**Digit rendering is a pixel grid, not character art** (`digits.rs`). Glyphs
are 10-pixel-row bitmaps (digits 8 px wide; colon/dot/space 4; minus 6). A
character cell is two pixels tall (`▀`/`▄`/`█`), and a cell is ~1:2, so pixels
are square; integer scaling happens in pixel space (character-level repetition
would corrupt half-blocks). Two layouts: `render` centers the digits alone and
hangs a minus in the left margin (timer negatives shift nothing);
`render_with_suffix` composes digits plus a half-scale suffix (stopwatch
milliseconds) and centers the whole unit. Both fall back to a plain text line
when scale 0.

**Time semantics.** `Timer` and `Stopwatch` both use accumulated `Duration`
plus `Option<Instant>` (drift-free pause). Timer display uses ceiling
(`(ms + 999).div_euclid(1000)`) so each value — including `00:00` — holds for
exactly one second; the stopwatch uses floor. `time_format.rs` holds the shared
sign/fields formatting; the hours field appears and disappears at the hour
boundary in both directions.

**Input subtleties** (`application.rs::handle_key`): a `typing` guard keeps
`h`, `l`, and `m` as literal input on the duration screen (they are duration
characters or adjacent to them) while arrows still switch tabs; the help popup
swallows everything except its close keys and `Ctrl+C`. Tab labels and mouse
hit-boxes both derive from `interface.rs::tab_layout` so they cannot drift.

**Audio** (`audio.rs`): one `MixerDeviceSink` and two rodio `Player`s (chime,
music). The `_device_sink` field is load-bearing — dropping it silences
everything. rodio 0.22 API differs from older docs: `DeviceSinkBuilder` /
`Player`, not `OutputStream` / `Sink`; `Player::stop()` is terminal, use
`clear()` then `play()` to reuse. `main.rs` sets the `PIPEWIRE_ALSA` env
variable before any thread exists so mixers show the stream as "Clock"
(`application.name` and `media.name` must stay equal — pulsemixer appends the
media name only when it differs).

**Sound assets are synthesized, not sourced**: `examples/generate_chime.rs`
and `examples/generate_elevator_music.rs` emit WAV bytes by hand (44-byte RIFF
header, no audio-file dependencies). `assets/chime.wav` is committed and
embedded via `include_bytes!`; regenerate it through the example rather than
editing the binary file.
