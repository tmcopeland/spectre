//! Raw bindings to the subset of the libsndfile C API that Spectre uses.
//!
//! These are hand-written against `sndfile.h` (libsndfile >= 1.0.25) so that the
//! build only needs the shared library, not the development headers or bindgen.
//! Everything here is `unsafe` to call; see the parent module for the safe API.

#![allow(non_camel_case_types, dead_code)]

use std::os::raw::{c_char, c_int, c_void};

/// Opaque libsndfile handle (`SNDFILE`).
#[allow(clippy::upper_case_acronyms)]
#[repr(C)]
pub struct SNDFILE {
    _private: [u8; 0],
}

pub type sf_count_t = i64;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SF_INFO {
    pub frames: sf_count_t,
    pub samplerate: c_int,
    pub channels: c_int,
    pub format: c_int,
    pub sections: c_int,
    pub seekable: c_int,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SF_FORMAT_INFO {
    pub format: c_int,
    pub name: *const c_char,
    pub extension: *const c_char,
}

// Open modes.
pub const SFM_READ: c_int = 0x10;
pub const SFM_WRITE: c_int = 0x20;

// Format masks.
pub const SF_FORMAT_SUBMASK: c_int = 0x0000_FFFF;
pub const SF_FORMAT_TYPEMASK: c_int = 0x0FFF_0000;
pub const SF_FORMAT_ENDMASK: c_int = 0x3000_0000;

// Major formats that Spectre refers to by name.
pub const SF_FORMAT_WAV: c_int = 0x0001_0000;
pub const SF_FORMAT_FLAC: c_int = 0x0017_0000;
pub const SF_FORMAT_OGG: c_int = 0x0020_0000;

// Subtypes that Spectre refers to by name.
pub const SF_FORMAT_PCM_16: c_int = 0x0002;
pub const SF_FORMAT_FLOAT: c_int = 0x0006;
pub const SF_FORMAT_VORBIS: c_int = 0x0060;

// Endianness flags.
pub const SF_ENDIAN_FILE: c_int = 0x0000_0000;
pub const SF_ENDIAN_LITTLE: c_int = 0x1000_0000;
pub const SF_ENDIAN_BIG: c_int = 0x2000_0000;
pub const SF_ENDIAN_CPU: c_int = 0x3000_0000;

// sf_command() commands.
pub const SFC_GET_LIB_VERSION: c_int = 0x1000;
pub const SFC_GET_FORMAT_INFO: c_int = 0x1028;
pub const SFC_GET_FORMAT_MAJOR_COUNT: c_int = 0x1030;
pub const SFC_GET_FORMAT_MAJOR: c_int = 0x1031;
pub const SFC_GET_FORMAT_SUBTYPE_COUNT: c_int = 0x1032;
pub const SFC_GET_FORMAT_SUBTYPE: c_int = 0x1033;

// sf_get_string() selectors.
pub const SF_STR_TITLE: c_int = 0x01;
pub const SF_STR_COPYRIGHT: c_int = 0x02;
pub const SF_STR_SOFTWARE: c_int = 0x03;
pub const SF_STR_ARTIST: c_int = 0x04;
pub const SF_STR_COMMENT: c_int = 0x05;
pub const SF_STR_DATE: c_int = 0x06;
pub const SF_STR_ALBUM: c_int = 0x07;

extern "C" {
    pub fn sf_open(path: *const c_char, mode: c_int, sfinfo: *mut SF_INFO) -> *mut SNDFILE;
    pub fn sf_close(sndfile: *mut SNDFILE) -> c_int;
    pub fn sf_error(sndfile: *mut SNDFILE) -> c_int;
    pub fn sf_strerror(sndfile: *mut SNDFILE) -> *const c_char;
    pub fn sf_format_check(info: *const SF_INFO) -> c_int;
    pub fn sf_command(
        sndfile: *mut SNDFILE,
        command: c_int,
        data: *mut c_void,
        datasize: c_int,
    ) -> c_int;
    pub fn sf_get_string(sndfile: *mut SNDFILE, str_type: c_int) -> *const c_char;
    pub fn sf_readf_float(sndfile: *mut SNDFILE, ptr: *mut f32, frames: sf_count_t) -> sf_count_t;
    pub fn sf_writef_float(
        sndfile: *mut SNDFILE,
        ptr: *const f32,
        frames: sf_count_t,
    ) -> sf_count_t;
    pub fn sf_write_sync(sndfile: *mut SNDFILE);
}
