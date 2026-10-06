# Spectre

An acoustic spectrum analyzer, written in Rust. Audio decoding is done by the
[libsndfile](https://libsndfile.github.io/libsndfile/) C library, so Spectre can
analyze whatever formats your libsndfile build understands: WAV, AIFF, FLAC,
Ogg Vorbis/Opus, MP3, CAF, W64, RF64 and many more (`spectre formats` lists them).
Formats libsndfile does not implement, such as AAC/M4A and WMA, are not supported.

This is a ground-up rewrite. The earlier C++/gtkmm/libav attempt is preserved in
the git history on `master`.

## Building

You need a Rust toolchain and the libsndfile **development** package:

```sh
sudo apt install libsndfile1-dev pkg-config   # Debian/Ubuntu
brew install libsndfile pkg-config            # macOS
cargo build --release
```

The build script finds the library via `pkg-config`, falling back to a plain
`-lsndfile`. If libsndfile lives somewhere unusual, set `SNDFILE_LIB_DIR` to the
directory containing it.

### Graphical interface

The GUI is built with [Slint](https://slint.dev) and is optional, so the command-line
build stays small. Building it needs Rust 1.92 or newer and, on Linux, the fontconfig
development package (`libfontconfig1-dev`):

```sh
cargo run --release --features gui --bin spectre-gui -- [FILE]
```

On Linux the window needs `libxkbcommon-x11`, plus a running X11 or Wayland session.
Slint is used under its GPL-3.0 option, which is compatible with this project's licence.

## Usage

```sh
spectre info song.flac                      # format, rate, channels, duration, tags
spectre render song.flac -o song.png        # spectrogram image
spectre render song.ogg -n 4096 -w 1600 -H 800 --window blackman -r 100
spectre render song.wav --channel 1         # right channel only (default: mono mix)
spectre formats                             # what this libsndfile build supports
```

Levels are in dBFS (0 dB is a full-scale sine); `--range` sets how many dB below
that are shown. Time runs left to right, frequency bottom (0 Hz) to top (Nyquist).
Files are streamed through libsndfile, so memory use does not grow with length.

### GUI

`spectre-gui` opens a window with the spectrogram, labelled time, frequency (kHz) and
level (dBFS) axes, and a crosshair whose frequency, time and level appear in the
status line. The toolbar has the FFT size, window function, channel and dynamic range;
changing the first three re-analyzes the file in the background, while the range
slider only recolours. Use **Open…** or pass a file on the command line.
Dropping files onto the window is not supported yet.

## Layout

| Path                    | Purpose                                                              |
|-------------------------|----------------------------------------------------------------------|
| `src/sndfile/ffi.rs`    | Hand-written `extern "C"` bindings (no headers or bindgen required)   |
| `src/sndfile/mod.rs`    | Safe `Reader`/`Writer`, format enumeration, tags, error handling      |
| `src/axis.rs`           | Axis tick generation and time/frequency label formatting              |
| `src/dsp.rs`            | Window functions                                                      |
| `src/spectrogram.rs`    | Streaming STFT, colormap, PNG rendering                               |
| `src/main.rs`           | Command-line interface                                                |
| `src/bin/spectre-gui.rs`, `ui/spectre.slint` | Slint GUI (`gui` feature)                        |
| `tests/analysis.rs`     | Encodes fixtures through libsndfile, then checks the analysis         |

## License

GPL-3.0-or-later; see `COPYING`.
