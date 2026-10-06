//! End-to-end tests: encode audio with libsndfile in several formats, then decode
//! and analyze it through the same C library.

use std::f32::consts::TAU;
use std::fs;
use std::path::PathBuf;

use spectre::dsp::Window;
use spectre::sndfile::{self, Format, Reader, Writer};
use spectre::spectrogram::{self, Channel, Params, RenderOptions};

const RATE: u32 = 44100;
const FFT: usize = 2048;
/// A frequency that falls exactly on FFT bin 48, so amplitude checks are not
/// affected by scalloping loss.
const BIN_CENTERED_HZ: f32 = 48.0 * RATE as f32 / FFT as f32;

fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// Interleaved sine waves, one frequency per channel, all at `amp`.
fn tones(freqs: &[f32], seconds: f32, amp: f32) -> Vec<f32> {
    let frames = (RATE as f32 * seconds) as usize;
    let mut out = Vec::with_capacity(frames * freqs.len());
    for i in 0..frames {
        for f in freqs {
            out.push(amp * (TAU * f * i as f32 / RATE as f32).sin());
        }
    }
    out
}

fn write(name: &str, format: Format, freqs: &[f32], seconds: f32) -> PathBuf {
    let path = tmp(name);
    let mut w = Writer::create(&path, format, RATE, freqs.len() as u32).unwrap();
    w.write_frames(&tones(freqs, seconds, 0.5)).unwrap();
    w.finish().unwrap();
    path
}

fn analyze(path: &PathBuf, channel: Channel) -> spectrogram::Spectrogram {
    let mut r = Reader::open(path).unwrap();
    let params = Params {
        fft_size: FFT,
        width: 64,
        window: Window::Hann,
        channel,
    };
    spectrogram::analyze(&mut r, &params).unwrap()
}

fn has_major(major: i32) -> bool {
    sndfile::formats()
        .iter()
        .any(|f| f.info.format.major() == major)
}

#[test]
fn wav_sine_peaks_at_the_right_frequency_and_level() {
    let path = write(
        "sine.wav",
        Format::new(sndfile::FORMAT_WAV, sndfile::SUBTYPE_FLOAT),
        &[BIN_CENTERED_HZ],
        1.5,
    );
    let spec = analyze(&path, Channel::Mix);
    let (hz, db) = spec.peak().unwrap();
    assert!(
        (hz - f64::from(BIN_CENTERED_HZ)).abs() < 1.0,
        "peak at {hz} Hz"
    );
    // Amplitude 0.5 is -6.02 dBFS.
    assert!((db + 6.02).abs() < 0.2, "peak level {db} dBFS");
}

#[test]
fn reader_reports_stream_info() {
    let path = write(
        "info.wav",
        Format::new(sndfile::FORMAT_WAV, sndfile::SUBTYPE_PCM_16),
        &[440.0, 880.0],
        1.0,
    );
    let r = Reader::open(&path).unwrap();
    let i = r.info();
    assert_eq!(
        (i.sample_rate, i.channels, i.frames),
        (RATE, 2, u64::from(RATE))
    );
    assert_eq!(i.format.major(), sndfile::FORMAT_WAV);
    assert_eq!(i.format.subtype(), sndfile::SUBTYPE_PCM_16);
    assert!(i.format.to_string().contains("WAV"), "{}", i.format);
}

#[test]
fn analyzes_other_containers_and_codecs() {
    // Lossless formats must agree with the WAV result; the lossy one only needs
    // to land on the right frequency.
    let cases: [(&str, i32, i32, bool); 3] = [
        (
            "sine.flac",
            sndfile::FORMAT_FLAC,
            sndfile::SUBTYPE_PCM_16,
            true,
        ),
        ("sine.aiff", 0x0002_0000, sndfile::SUBTYPE_PCM_16, true),
        (
            "sine.ogg",
            sndfile::FORMAT_OGG,
            sndfile::SUBTYPE_VORBIS,
            false,
        ),
    ];
    let mut ran = 0;
    for (name, major, sub, lossless) in cases {
        if !has_major(major) {
            eprintln!("skipping {name}: not supported by this libsndfile build");
            continue;
        }
        let path = write(name, Format::new(major, sub), &[BIN_CENTERED_HZ], 1.5);
        let (hz, db) = analyze(&path, Channel::Mix).peak().unwrap();
        assert!(
            (hz - f64::from(BIN_CENTERED_HZ)).abs() < 1.0,
            "{name}: peak at {hz} Hz"
        );
        let tol = if lossless { 0.2 } else { 1.5 };
        assert!((db + 6.02).abs() < tol, "{name}: peak level {db} dBFS");
        ran += 1;
    }
    assert!(ran > 0, "no optional formats were available to test");
}

#[test]
fn selects_individual_channels() {
    let (left, right) = (
        48.0 * RATE as f32 / FFT as f32,
        192.0 * RATE as f32 / FFT as f32,
    );
    let path = write(
        "stereo.wav",
        Format::new(sndfile::FORMAT_WAV, sndfile::SUBTYPE_FLOAT),
        &[left, right],
        1.0,
    );

    let hz = |ch| analyze(&path, ch).peak().unwrap().0;
    assert!((hz(Channel::Index(0)) - f64::from(left)).abs() < 1.0);
    assert!((hz(Channel::Index(1)) - f64::from(right)).abs() < 1.0);

    // The mix contains both tones at half amplitude each (-12 dBFS).
    let mix = analyze(&path, Channel::Mix);
    let bin = |hz: f32| (hz / mix.bin_hz() as f32).round() as usize;
    let avg =
        |b: usize| mix.data.iter().skip(b).step_by(mix.bins).sum::<f32>() / mix.columns as f32;
    assert!((avg(bin(left)) + 12.04).abs() < 0.3, "{}", avg(bin(left)));
    assert!((avg(bin(right)) + 12.04).abs() < 0.3, "{}", avg(bin(right)));
}

#[test]
fn rejects_out_of_range_channel() {
    let path = write(
        "mono.wav",
        Format::new(sndfile::FORMAT_WAV, sndfile::SUBTYPE_PCM_16),
        &[440.0],
        0.5,
    );
    let mut r = Reader::open(&path).unwrap();
    let params = Params {
        channel: Channel::Index(1),
        ..Params::default()
    };
    let err = spectrogram::analyze(&mut r, &params).unwrap_err();
    assert!(err.to_string().contains("channel 1"), "{err}");
}

#[test]
fn rejects_bad_fft_size() {
    let path = write(
        "fft.wav",
        Format::new(sndfile::FORMAT_WAV, sndfile::SUBTYPE_PCM_16),
        &[440.0],
        0.5,
    );
    let mut r = Reader::open(&path).unwrap();
    let params = Params {
        fft_size: 1000,
        ..Params::default()
    };
    assert!(spectrogram::analyze(&mut r, &params).is_err());
}

#[test]
fn files_shorter_than_the_requested_width_still_work() {
    let path = tmp("tiny.wav");
    let mut w = Writer::create(
        &path,
        Format::new(sndfile::FORMAT_WAV, sndfile::SUBTYPE_FLOAT),
        RATE,
        1,
    )
    .unwrap();
    w.write_frames(&tones(&[1000.0], 0.001, 0.5)).unwrap(); // 44 frames
    w.finish().unwrap();
    let mut r = Reader::open(&path).unwrap();
    let spec = spectrogram::analyze(
        &mut r,
        &Params {
            width: 500,
            ..Params::default()
        },
    )
    .unwrap();
    assert_eq!(spec.columns, 44);
    assert!(spec.data.iter().all(|v| v.is_finite()));
}

#[test]
fn digital_silence_is_finite() {
    let path = tmp("silence.wav");
    let mut w = Writer::create(
        &path,
        Format::new(sndfile::FORMAT_WAV, sndfile::SUBTYPE_PCM_16),
        RATE,
        1,
    )
    .unwrap();
    w.write_frames(&vec![0.0; 22050]).unwrap();
    w.finish().unwrap();
    let spec = analyze(&path, Channel::Mix);
    assert!(spec.data.iter().all(|v| v.is_finite() && *v <= -200.0));
}

#[test]
fn missing_and_non_audio_files_report_errors() {
    assert!(Reader::open(tmp("does-not-exist.wav")).is_err());

    let junk = tmp("junk.wav");
    fs::write(&junk, b"this is definitely not audio data, just some text").unwrap();
    let err = Reader::open(&junk).err().expect("junk must not open");
    assert!(!err.to_string().is_empty());
}

#[test]
fn renders_a_png_of_the_requested_size() {
    let path = write(
        "render.wav",
        Format::new(sndfile::FORMAT_WAV, sndfile::SUBTYPE_FLOAT),
        &[BIN_CENTERED_HZ],
        1.0,
    );
    let spec = analyze(&path, Channel::Mix);
    let out = tmp("render.png");
    spectrogram::write_png(
        &spec,
        &RenderOptions {
            height: Some(256),
            range_db: 100.0,
        },
        &out,
    )
    .unwrap();

    let decoder = png::Decoder::new(std::io::BufReader::new(fs::File::open(&out).unwrap()));
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let frame = reader.next_frame(&mut buf).unwrap();
    assert_eq!((frame.width, frame.height), (64, 256));

    // The tone sits at ~1.03 kHz of 22.05 kHz: a bright pixel ~95% of the way down, and
    // the top row (near Nyquist) stays black.
    let px = |x: usize, y: usize| &buf[(y * 64 + x) * 3..(y * 64 + x) * 3 + 3];
    assert_eq!(px(32, 0), &[0, 0, 0]);
    let brightest = (0..256)
        .max_by_key(|&y| px(32, y).iter().map(|&c| u32::from(c)).sum::<u32>())
        .unwrap();
    let expect = 256.0 * (1.0 - f64::from(BIN_CENTERED_HZ) / 22050.0);
    assert!(
        (brightest as f64 - expect).abs() < 3.0,
        "brightest row {brightest}, expected ~{expect}"
    );
}

#[test]
fn lists_formats_including_wav() {
    let majors = sndfile::formats();
    let wav = majors
        .iter()
        .find(|f| f.info.format.major() == sndfile::FORMAT_WAV)
        .expect("WAV");
    assert!(wav
        .subtypes
        .iter()
        .any(|s| s.format.0 == sndfile::SUBTYPE_PCM_16));
    assert!(sndfile::lib_version().starts_with("libsndfile"));
}
