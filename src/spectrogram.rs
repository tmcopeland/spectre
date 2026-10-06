//! Short-time Fourier transform over a libsndfile stream, plus PNG rendering.

use std::fmt;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use realfft::RealFftPlanner;

use crate::dsp::Window;
use crate::sndfile::{self, Reader};

/// Which audio channel(s) to analyze.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// Average of all channels.
    Mix,
    /// A single zero-based channel.
    Index(u32),
}

#[derive(Debug, Clone)]
pub struct Params {
    /// FFT length in samples; must be a power of two >= 16.
    pub fft_size: usize,
    /// Number of time columns to produce (fewer if the file has fewer frames).
    pub width: usize,
    pub window: Window,
    pub channel: Channel,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            fft_size: 2048,
            width: 1024,
            window: Window::Hann,
            channel: Channel::Mix,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Audio(sndfile::Error),
    Io(std::io::Error),
    Encode(png::EncodingError),
    Invalid(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Audio(e) => write!(f, "{e}"),
            Error::Io(e) => write!(f, "{e}"),
            Error::Encode(e) => write!(f, "{e}"),
            Error::Invalid(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<sndfile::Error> for Error {
    fn from(e: sndfile::Error) -> Self {
        Error::Audio(e)
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}
impl From<png::EncodingError> for Error {
    fn from(e: png::EncodingError) -> Self {
        Error::Encode(e)
    }
}

/// Spectrogram magnitudes in dBFS (0 dB = full-scale sine).
#[derive(Debug, Clone)]
pub struct Spectrogram {
    /// Frequency bins per column (`fft_size / 2 + 1`).
    pub bins: usize,
    pub columns: usize,
    /// Column-major: `data[column * bins + bin]`.
    pub data: Vec<f32>,
    pub sample_rate: u32,
    /// Length of the analyzed audio in seconds.
    pub duration_secs: f64,
}

/// One spectrogram cell and where it sits in time and frequency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub time_secs: f64,
    pub freq_hz: f64,
    pub db: f32,
}

/// Lowest level we report; avoids -inf for digital silence.
const FLOOR_DB: f32 = -240.0;

impl Spectrogram {
    pub fn column(&self, c: usize) -> &[f32] {
        &self.data[c * self.bins..(c + 1) * self.bins]
    }

    /// Width of one frequency bin in Hz.
    pub fn bin_hz(&self) -> f64 {
        f64::from(self.sample_rate) / 2.0 / (self.bins - 1) as f64
    }

    /// Looks up the cell under a point given as fractions of the plot area:
    /// `x` from the left edge, `y` from the bottom edge. Returns `None` outside `0..=1`.
    pub fn at(&self, x: f64, y: f64) -> Option<Cell> {
        if !(0.0..=1.0).contains(&x)
            || !(0.0..=1.0).contains(&y)
            || self.columns == 0
            || self.bins < 2
        {
            return None;
        }
        let column = ((x * self.columns as f64) as usize).min(self.columns - 1);
        let bin = (y * (self.bins - 1) as f64).round() as usize;
        Some(Cell {
            time_secs: x * self.duration_secs,
            freq_hz: y * f64::from(self.sample_rate) / 2.0,
            db: self.column(column)[bin],
        })
    }

    /// Frequency and level of the loudest bin anywhere in the spectrogram (DC excluded).
    pub fn peak(&self) -> Option<(f64, f32)> {
        if self.columns == 0 || self.bins < 2 {
            return None;
        }
        self.data
            .chunks_exact(self.bins)
            .flat_map(|col| col.iter().enumerate().skip(1))
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(bin, db)| (bin as f64 * self.bin_hz(), *db))
    }
}

/// Pulls mono samples out of a [`Reader`], tracking absolute frame positions so
/// that arbitrary (monotonically advancing) windows can be served without seeking.
struct MonoStream<'a> {
    reader: &'a mut Reader,
    channel: Channel,
    scratch: Vec<f32>,
    buf: Vec<f32>,
    /// Absolute frame index of `buf[0]`.
    start: i64,
    eof: bool,
}

impl<'a> MonoStream<'a> {
    const CHUNK_FRAMES: usize = 8192;

    fn new(reader: &'a mut Reader, channel: Channel) -> Self {
        let channels = reader.info().channels as usize;
        MonoStream {
            reader,
            channel,
            scratch: vec![0.0; Self::CHUNK_FRAMES * channels],
            buf: Vec::new(),
            start: 0,
            eof: false,
        }
    }

    fn end(&self) -> i64 {
        self.start + self.buf.len() as i64
    }

    fn fill_until(&mut self, target_end: i64) -> Result<(), Error> {
        let channels = self.reader.info().channels as usize;
        while !self.eof && self.end() < target_end {
            let n = self.reader.read_frames(&mut self.scratch)?;
            if n == 0 {
                self.eof = true;
                break;
            }
            for frame in self.scratch[..n * channels].chunks_exact(channels) {
                self.buf.push(match self.channel {
                    Channel::Mix => frame.iter().sum::<f32>() / channels as f32,
                    Channel::Index(c) => frame[c as usize],
                });
            }
        }
        Ok(())
    }

    /// Fills `out` with frames `[start, start + out.len())`, zero-padding outside the file.
    /// `start` must not move backwards between calls (beyond what was already discarded).
    fn window_at(&mut self, start: i64, out: &mut [f32]) -> Result<(), Error> {
        let end = start + out.len() as i64;
        self.fill_until(end)?;
        // Discard what can no longer be needed.
        let drop = (start - self.start).clamp(0, self.buf.len() as i64) as usize;
        self.buf.drain(..drop);
        self.start += drop as i64;

        for (i, slot) in out.iter_mut().enumerate() {
            let idx = start + i as i64 - self.start;
            *slot = if idx >= 0 {
                self.buf.get(idx as usize).copied().unwrap_or(0.0)
            } else {
                0.0
            };
        }
        Ok(())
    }
}

/// Computes a spectrogram of the whole file by streaming it through libsndfile.
pub fn analyze(reader: &mut Reader, params: &Params) -> Result<Spectrogram, Error> {
    let n = params.fft_size;
    if !n.is_power_of_two() || n < 16 {
        return Err(Error::Invalid(format!(
            "FFT size must be a power of two >= 16, got {n}"
        )));
    }
    if params.width == 0 {
        return Err(Error::Invalid("width must be at least 1".into()));
    }
    let info = *reader.info();
    if let Channel::Index(c) = params.channel {
        if c >= info.channels {
            return Err(Error::Invalid(format!(
                "channel {c} requested but the file has {} channel(s)",
                info.channels
            )));
        }
    }
    if info.frames == 0 {
        return Err(Error::Invalid("file reports no audio frames".into()));
    }

    let total = info.frames as i64;
    let columns = params.width.min(total as usize);
    let hop = total as f64 / columns as f64;

    let window = params.window.coefficients(n);
    // A full-scale sine has FFT magnitude sum(w) / 2 at its bin.
    let norm = 2.0 / window.iter().sum::<f32>();

    let r2c = RealFftPlanner::<f32>::new().plan_fft_forward(n);
    let mut input = r2c.make_input_vec();
    let mut spectrum = r2c.make_output_vec();
    let mut scratch = r2c.make_scratch_vec();
    let bins = n / 2 + 1;

    let mut stream = MonoStream::new(reader, params.channel);
    let mut data = Vec::with_capacity(columns * bins);

    for col in 0..columns {
        let center = ((col as f64 + 0.5) * hop) as i64;
        stream.window_at(center - (n / 2) as i64, &mut input)?;
        for (s, w) in input.iter_mut().zip(&window) {
            *s *= w;
        }
        r2c.process_with_scratch(&mut input, &mut spectrum, &mut scratch)
            .map_err(|e| Error::Invalid(format!("FFT failed: {e}")))?;
        data.extend(
            spectrum
                .iter()
                .map(|c| (20.0 * (c.norm() * norm).log10()).max(FLOOR_DB)),
        );
    }

    Ok(Spectrogram {
        bins,
        columns,
        data,
        sample_rate: info.sample_rate,
        duration_secs: info.duration_secs(),
    })
}

/// Maps `t` in [0, 1] to a color: black → blue → purple → red → orange → yellow → white.
pub fn colormap(t: f32) -> [u8; 3] {
    const STOPS: [(f32, [f32; 3]); 7] = [
        (0.00, [0.0, 0.0, 0.0]),
        (0.15, [0.0, 0.0, 0.55]),
        (0.35, [0.5, 0.0, 0.6]),
        (0.55, [0.85, 0.1, 0.1]),
        (0.75, [1.0, 0.6, 0.0]),
        (0.90, [1.0, 1.0, 0.2]),
        (1.00, [1.0, 1.0, 1.0]),
    ];
    let t = t.clamp(0.0, 1.0);
    let hi = STOPS
        .iter()
        .position(|(p, _)| *p >= t)
        .unwrap_or(STOPS.len() - 1)
        .max(1);
    let (p0, c0) = STOPS[hi - 1];
    let (p1, c1) = STOPS[hi];
    let f = (t - p0) / (p1 - p0);
    let mut rgb = [0u8; 3];
    for i in 0..3 {
        rgb[i] = ((c0[i] + (c1[i] - c0[i]) * f) * 255.0).round() as u8;
    }
    rgb
}

#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    /// Image height in pixels; `None` uses one pixel per FFT bin.
    pub height: Option<usize>,
    /// Dynamic range shown below 0 dBFS.
    pub range_db: f32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            height: None,
            range_db: 120.0,
        }
    }
}

/// Rasterizes the spectrogram to RGB pixels (row-major, highest frequency at the top).
/// Returns `(width, height, pixels)`.
pub fn rasterize(spec: &Spectrogram, opts: &RenderOptions) -> (usize, usize, Vec<u8>) {
    let width = spec.columns;
    let height = opts.height.unwrap_or(spec.bins - 1).max(1);
    let mut pixels = vec![0u8; width * height * 3];
    let last = spec.bins - 1;

    // Per-row source range: when shrinking take the loudest bin (so narrow peaks
    // survive), when enlarging interpolate between neighbours.
    let level = |col: &[f32], row: usize| -> f32 {
        let from_bottom = height - 1 - row;
        if height <= last {
            let lo = from_bottom * last / height;
            let hi = (((from_bottom + 1) * last).div_ceil(height)).min(last);
            col[lo..=hi]
                .iter()
                .copied()
                .fold(f32::NEG_INFINITY, f32::max)
        } else {
            let pos = (from_bottom as f32 + 0.5) / height as f32 * last as f32;
            let lo = pos.floor() as usize;
            let hi = (lo + 1).min(last);
            let f = pos - lo as f32;
            col[lo] * (1.0 - f) + col[hi] * f
        }
    };

    for x in 0..width {
        let col = spec.column(x);
        for y in 0..height {
            let t = (level(col, y) + opts.range_db) / opts.range_db;
            let o = (y * width + x) * 3;
            pixels[o..o + 3].copy_from_slice(&colormap(t));
        }
    }
    (width, height, pixels)
}

/// Renders the spectrogram as an RGB PNG.
pub fn write_png(spec: &Spectrogram, opts: &RenderOptions, path: &Path) -> Result<(), Error> {
    let (w, h, pixels) = rasterize(spec, opts);
    let file = BufWriter::new(File::create(path)?);
    let mut enc = png::Encoder::new(file, w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(&pixels)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colormap_endpoints() {
        assert_eq!(colormap(0.0), [0, 0, 0]);
        assert_eq!(colormap(1.0), [255, 255, 255]);
        assert_eq!(colormap(-5.0), [0, 0, 0]);
        assert_eq!(colormap(5.0), [255, 255, 255]);
    }

    #[test]
    fn at_maps_plot_fractions_to_cells() {
        // Two columns, three bins at 0, 2 kHz and 4 kHz over 2 seconds.
        let spec = Spectrogram {
            bins: 3,
            columns: 2,
            data: vec![-10.0, -20.0, -30.0, -40.0, -50.0, -60.0],
            sample_rate: 8000,
            duration_secs: 2.0,
        };
        let top_right = spec.at(1.0, 1.0).unwrap();
        assert_eq!(
            (top_right.time_secs, top_right.freq_hz, top_right.db),
            (2.0, 4000.0, -60.0)
        );
        let bottom_left = spec.at(0.0, 0.0).unwrap();
        assert_eq!(
            (bottom_left.time_secs, bottom_left.freq_hz, bottom_left.db),
            (0.0, 0.0, -10.0)
        );
        assert_eq!(spec.at(0.6, 0.5).unwrap().db, -50.0);
        assert!(spec.at(-0.01, 0.5).is_none() && spec.at(0.5, 1.01).is_none());
    }

    #[test]
    fn rasterize_puts_high_frequencies_on_top() {
        // One column, 5 bins; only the top bin is loud.
        let spec = Spectrogram {
            bins: 5,
            columns: 1,
            data: vec![-120.0, -120.0, -120.0, -120.0, 0.0],
            sample_rate: 8000,
            duration_secs: 1.0,
        };
        let (w, h, px) = rasterize(
            &spec,
            &RenderOptions {
                height: Some(4),
                range_db: 120.0,
            },
        );
        assert_eq!((w, h), (1, 4));
        assert_eq!(&px[0..3], &[255, 255, 255]); // top row = Nyquist
        assert_eq!(&px[9..12], &[0, 0, 0]); // bottom row = DC
    }
}
