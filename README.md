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

## Layout

| Path                    | Purpose                                                              |
|-------------------------|----------------------------------------------------------------------|
| `src/sndfile/ffi.rs`    | Hand-written `extern "C"` bindings (no headers or bindgen required)   |
| `src/sndfile/mod.rs`    | Safe `Reader`/`Writer`, format enumeration, tags, error handling      |
| `src/dsp.rs`            | Window functions                                                      |
| `src/spectrogram.rs`    | Streaming STFT, colormap, PNG rendering                               |
| `src/main.rs`           | Command-line interface                                                |
| `tests/analysis.rs`     | Encodes fixtures through libsndfile, then checks the analysis         |

## License

GPL-3.0-or-later; see `COPYING`.
