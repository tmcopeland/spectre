//! Spectre: an acoustic spectrum analyzer.
//!
//! Audio decoding is delegated to the libsndfile C library ([`sndfile`]), the
//! signal processing lives in [`dsp`] and [`spectrogram`].

pub mod axis;
pub mod dsp;
pub mod sndfile;
pub mod spectrogram;
