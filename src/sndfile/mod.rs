//! Safe wrapper around libsndfile.
//!
//! libsndfile decodes dozens of container/codec combinations (WAV, AIFF, FLAC,
//! Ogg Vorbis/Opus, MP3, CAF, W64, ...) behind a single API. The exact set depends
//! on how the installed library was built; [`formats`] reports what is available.

mod ffi;

use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr::{self, NonNull};

pub use ffi::{
    SF_FORMAT_FLAC as FORMAT_FLAC, SF_FORMAT_OGG as FORMAT_OGG, SF_FORMAT_WAV as FORMAT_WAV,
};
pub use ffi::{
    SF_FORMAT_FLOAT as SUBTYPE_FLOAT, SF_FORMAT_PCM_16 as SUBTYPE_PCM_16,
    SF_FORMAT_VORBIS as SUBTYPE_VORBIS,
};

#[derive(Debug)]
pub enum Error {
    /// The path contains an interior NUL byte and cannot be passed to C.
    InvalidPath,
    /// libsndfile reported an error (message from `sf_strerror`).
    Sndfile(String),
    /// The file has no channels or an unusable sample rate.
    BadStream(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidPath => write!(f, "path contains a NUL byte"),
            Error::Sndfile(msg) => write!(f, "{msg}"),
            Error::BadStream(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// Copies a possibly-NULL C string owned by libsndfile into a Rust `String`.
///
/// # Safety
/// `ptr` must be NULL or point at a valid NUL-terminated string.
unsafe fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
    }
}

/// libsndfile's "last error" for calls that have no handle (e.g. a failed `sf_open`).
fn global_error() -> Error {
    // SAFETY: sf_strerror(NULL) returns a pointer to a static/thread-global buffer.
    let msg = unsafe { cstr_to_string(ffi::sf_strerror(ptr::null_mut())) };
    Error::Sndfile(msg.unwrap_or_else(|| "unknown libsndfile error".into()))
}

/// The version string of the loaded libsndfile, e.g. `"libsndfile-1.2.2"`.
pub fn lib_version() -> String {
    let mut buf = [0 as c_char; 128];
    // SAFETY: buf is valid for 128 bytes; the command writes a NUL-terminated string.
    let n = unsafe {
        ffi::sf_command(
            ptr::null_mut(),
            ffi::SFC_GET_LIB_VERSION,
            buf.as_mut_ptr() as *mut c_void,
            buf.len() as c_int,
        )
    };
    if n <= 0 {
        return "unknown".into();
    }
    buf[buf.len() - 1] = 0;
    unsafe { cstr_to_string(buf.as_ptr()) }.unwrap_or_else(|| "unknown".into())
}

/// A libsndfile format word: major container | subtype | endianness.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Format(pub i32);

impl Format {
    pub fn new(major: i32, subtype: i32) -> Self {
        Format(major | subtype)
    }

    pub fn major(self) -> i32 {
        self.0 & ffi::SF_FORMAT_TYPEMASK
    }

    pub fn subtype(self) -> i32 {
        self.0 & ffi::SF_FORMAT_SUBMASK
    }

    /// Human-readable container name, e.g. `"WAV (Microsoft)"`.
    pub fn major_name(self) -> Option<String> {
        format_info(self.major()).map(|i| i.name)
    }

    /// Human-readable codec/sample-type name, e.g. `"Signed 16 bit PCM"`.
    pub fn subtype_name(self) -> Option<String> {
        format_info(self.subtype()).map(|i| i.name)
    }

    pub fn endianness(self) -> Option<&'static str> {
        match self.0 & ffi::SF_FORMAT_ENDMASK {
            ffi::SF_ENDIAN_LITTLE => Some("little"),
            ffi::SF_ENDIAN_BIG => Some("big"),
            _ => None,
        }
    }
}

impl fmt::Debug for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Format({:#010x})", self.0)
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let major = self
            .major_name()
            .unwrap_or_else(|| "unknown container".into());
        let sub = self
            .subtype_name()
            .unwrap_or_else(|| "unknown codec".into());
        write!(f, "{major}, {sub}")
    }
}

/// Name and typical file extension for one major format or subtype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatInfo {
    pub format: Format,
    pub name: String,
    pub extension: Option<String>,
}

fn convert_info(raw: &ffi::SF_FORMAT_INFO) -> FormatInfo {
    // SAFETY: libsndfile fills these with pointers to static strings (or NULL).
    unsafe {
        FormatInfo {
            format: Format(raw.format),
            name: cstr_to_string(raw.name).unwrap_or_default(),
            extension: cstr_to_string(raw.extension),
        }
    }
}

/// Looks up the description of a bare major format or bare subtype value.
fn format_info(format: i32) -> Option<FormatInfo> {
    let mut raw = ffi::SF_FORMAT_INFO {
        format,
        name: ptr::null(),
        extension: ptr::null(),
    };
    // SAFETY: raw is a valid SF_FORMAT_INFO and the size matches.
    let rc = unsafe {
        ffi::sf_command(
            ptr::null_mut(),
            ffi::SFC_GET_FORMAT_INFO,
            &mut raw as *mut _ as *mut c_void,
            std::mem::size_of::<ffi::SF_FORMAT_INFO>() as c_int,
        )
    };
    (rc == 0 && !raw.name.is_null()).then(|| convert_info(&raw))
}

fn enumerate(count_cmd: c_int, item_cmd: c_int) -> Vec<FormatInfo> {
    let mut count: c_int = 0;
    // SAFETY: count is a valid c_int destination.
    unsafe {
        ffi::sf_command(
            ptr::null_mut(),
            count_cmd,
            &mut count as *mut _ as *mut c_void,
            std::mem::size_of::<c_int>() as c_int,
        );
    }
    (0..count.max(0))
        .filter_map(|i| {
            // For enumeration commands, `format` carries the index on input.
            let mut raw = ffi::SF_FORMAT_INFO {
                format: i,
                name: ptr::null(),
                extension: ptr::null(),
            };
            let rc = unsafe {
                ffi::sf_command(
                    ptr::null_mut(),
                    item_cmd,
                    &mut raw as *mut _ as *mut c_void,
                    std::mem::size_of::<ffi::SF_FORMAT_INFO>() as c_int,
                )
            };
            (rc == 0 && !raw.name.is_null()).then(|| convert_info(&raw))
        })
        .collect()
}

/// A container format the loaded libsndfile can handle, with the subtypes valid in it.
#[derive(Debug, Clone)]
pub struct SupportedFormat {
    pub info: FormatInfo,
    pub subtypes: Vec<FormatInfo>,
}

/// Every container format supported by the loaded libsndfile build.
pub fn formats() -> Vec<SupportedFormat> {
    let majors = enumerate(ffi::SFC_GET_FORMAT_MAJOR_COUNT, ffi::SFC_GET_FORMAT_MAJOR);
    let subtypes = enumerate(
        ffi::SFC_GET_FORMAT_SUBTYPE_COUNT,
        ffi::SFC_GET_FORMAT_SUBTYPE,
    );
    majors
        .into_iter()
        .map(|info| {
            let valid = subtypes
                .iter()
                .filter(|s| {
                    let probe = ffi::SF_INFO {
                        channels: 1,
                        samplerate: 44100,
                        format: info.format.0 | s.format.0,
                        ..Default::default()
                    };
                    // SAFETY: probe is a valid SF_INFO.
                    unsafe { ffi::sf_format_check(&probe) != 0 }
                })
                .cloned()
                .collect();
            SupportedFormat {
                info,
                subtypes: valid,
            }
        })
        .collect()
}

/// Stream parameters reported by libsndfile.
#[derive(Debug, Clone, Copy)]
pub struct Info {
    /// Total frames (one sample per channel). May be 0 if the length is unknown.
    pub frames: u64,
    pub sample_rate: u32,
    pub channels: u32,
    pub format: Format,
    pub seekable: bool,
}

impl Info {
    pub fn duration_secs(&self) -> f64 {
        self.frames as f64 / f64::from(self.sample_rate)
    }
}

/// Textual tags embedded in the file; absent ones are `None`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub date: Option<String>,
    pub comment: Option<String>,
    pub software: Option<String>,
    pub copyright: Option<String>,
}

impl Tags {
    pub fn is_empty(&self) -> bool {
        *self == Tags::default()
    }
}

/// Owns an open `SNDFILE*` and closes it on drop.
struct Handle(NonNull<ffi::SNDFILE>);

// SAFETY: a SNDFILE handle may be moved between threads as long as it is not
// used from two threads at once, which `&mut self` on every operation guarantees.
unsafe impl Send for Handle {}

impl Handle {
    fn open(path: &Path, mode: c_int, info: &mut ffi::SF_INFO) -> Result<Handle> {
        let c_path = path_to_cstring(path)?;
        // SAFETY: c_path is NUL-terminated; info is a valid SF_INFO.
        let raw = unsafe { ffi::sf_open(c_path.as_ptr(), mode, info) };
        NonNull::new(raw).map(Handle).ok_or_else(global_error)
    }

    fn as_ptr(&self) -> *mut ffi::SNDFILE {
        self.0.as_ptr()
    }

    fn check(&self) -> Result<()> {
        // SAFETY: handle is live.
        let code = unsafe { ffi::sf_error(self.as_ptr()) };
        if code == 0 {
            return Ok(());
        }
        let msg = unsafe { cstr_to_string(ffi::sf_strerror(self.as_ptr())) };
        Err(Error::Sndfile(
            msg.unwrap_or_else(|| format!("libsndfile error {code}")),
        ))
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: handle is live and never used again. Close errors cannot be reported from drop.
        unsafe { ffi::sf_close(self.as_ptr()) };
    }
}

#[cfg(unix)]
fn path_to_cstring(path: &Path) -> Result<CString> {
    use std::os::unix::ffi::OsStrExt;
    CString::new(path.as_os_str().as_bytes()).map_err(|_| Error::InvalidPath)
}

#[cfg(not(unix))]
fn path_to_cstring(path: &Path) -> Result<CString> {
    CString::new(path.to_string_lossy().as_bytes()).map_err(|_| Error::InvalidPath)
}

/// An audio file opened for reading.
pub struct Reader {
    handle: Handle,
    info: Info,
}

impl Reader {
    /// Opens `path`, letting libsndfile detect the format from the file contents.
    pub fn open(path: impl AsRef<Path>) -> Result<Reader> {
        let mut raw = ffi::SF_INFO::default();
        let handle = Handle::open(path.as_ref(), ffi::SFM_READ, &mut raw)?;
        if raw.channels <= 0 {
            return Err(Error::BadStream("file has no audio channels"));
        }
        if raw.samplerate <= 0 {
            return Err(Error::BadStream("file has an invalid sample rate"));
        }
        let info = Info {
            frames: raw.frames.max(0) as u64,
            sample_rate: raw.samplerate as u32,
            channels: raw.channels as u32,
            format: Format(raw.format),
            seekable: raw.seekable != 0,
        };
        Ok(Reader { handle, info })
    }

    pub fn info(&self) -> &Info {
        &self.info
    }

    pub fn tags(&self) -> Tags {
        let get = |kind| {
            // SAFETY: handle is live; the returned string is owned by libsndfile.
            unsafe { cstr_to_string(ffi::sf_get_string(self.handle.as_ptr(), kind)) }
                .filter(|s| !s.is_empty())
        };
        Tags {
            title: get(ffi::SF_STR_TITLE),
            artist: get(ffi::SF_STR_ARTIST),
            album: get(ffi::SF_STR_ALBUM),
            date: get(ffi::SF_STR_DATE),
            comment: get(ffi::SF_STR_COMMENT),
            software: get(ffi::SF_STR_SOFTWARE),
            copyright: get(ffi::SF_STR_COPYRIGHT),
        }
    }

    /// Reads up to `buf.len() / channels` frames of interleaved samples, scaled to
    /// [-1.0, 1.0] for integer formats. Returns the number of frames read; 0 means EOF.
    pub fn read_frames(&mut self, buf: &mut [f32]) -> Result<usize> {
        let channels = self.info.channels as usize;
        let frames = (buf.len() / channels) as ffi::sf_count_t;
        // SAFETY: buf holds at least frames * channels floats.
        let n = unsafe { ffi::sf_readf_float(self.handle.as_ptr(), buf.as_mut_ptr(), frames) };
        // A short read is normal at EOF; only report a sticky error if nothing came back.
        if n == 0 {
            self.handle.check()?;
        }
        Ok(n.max(0) as usize)
    }
}

/// An audio file opened for writing. Mainly useful for producing test fixtures
/// in any format libsndfile can encode.
pub struct Writer {
    handle: Handle,
    channels: usize,
}

impl Writer {
    pub fn create(
        path: impl AsRef<Path>,
        format: Format,
        sample_rate: u32,
        channels: u32,
    ) -> Result<Writer> {
        let mut raw = ffi::SF_INFO {
            samplerate: sample_rate as c_int,
            channels: channels as c_int,
            format: format.0,
            ..Default::default()
        };
        // SAFETY: raw is a valid SF_INFO.
        if unsafe { ffi::sf_format_check(&raw) } == 0 {
            return Err(Error::Sndfile(format!(
                "{format} is not a valid combination for {channels} channel(s) at {sample_rate} Hz"
            )));
        }
        let handle = Handle::open(path.as_ref(), ffi::SFM_WRITE, &mut raw)?;
        Ok(Writer {
            handle,
            channels: channels as usize,
        })
    }

    /// Writes interleaved samples; `samples.len()` must be a multiple of the channel count.
    pub fn write_frames(&mut self, samples: &[f32]) -> Result<()> {
        assert_eq!(samples.len() % self.channels, 0, "partial frame");
        let frames = (samples.len() / self.channels) as ffi::sf_count_t;
        // SAFETY: samples holds frames * channels floats.
        let n = unsafe { ffi::sf_writef_float(self.handle.as_ptr(), samples.as_ptr(), frames) };
        if n != frames {
            self.handle.check()?;
            return Err(Error::Sndfile("short write".into()));
        }
        Ok(())
    }

    /// Flushes and closes the file, reporting any error from the final flush.
    pub fn finish(self) -> Result<()> {
        // SAFETY: handle is live.
        unsafe { ffi::sf_write_sync(self.handle.as_ptr()) };
        self.handle.check()
    }
}
