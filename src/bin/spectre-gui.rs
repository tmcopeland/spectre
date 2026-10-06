//! Graphical spectrogram viewer built with Slint.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use slint::{
    ComponentHandle, Image, ModelRc, Rgb8Pixel, SharedPixelBuffer, SharedString, VecModel,
};

use spectre::axis::{self, Tick};
use spectre::dsp::Window;
use spectre::sndfile::{self, Reader};
use spectre::spectrogram::{self, Channel, Params, RenderOptions, Spectrogram};

slint::include_modules!();

const FFT_SIZES: [usize; 5] = [512, 1024, 2048, 4096, 8192];
const WINDOWS: [Window; 4] = [
    Window::Hann,
    Window::Hamming,
    Window::Blackman,
    Window::Rectangular,
];
/// Time columns computed per analysis; the image is scaled to the window.
const COLUMNS: usize = 1600;
const MAX_IMAGE_HEIGHT: usize = 1024;
const COLORBAR_STEPS: usize = 256;

/// Result of analyzing one file.
struct Loaded {
    path: PathBuf,
    spec: Arc<Spectrogram>,
    summary: String,
    channels: u32,
    /// True when this came from opening a file rather than from changing settings.
    new_file: bool,
}

#[derive(Default)]
struct Shared {
    path: Option<PathBuf>,
    spec: Option<Arc<Spectrogram>>,
}

/// State shared between the UI thread and analysis workers.
#[derive(Clone, Default)]
struct Ctx {
    shared: Arc<Mutex<Shared>>,
    /// Incremented per analysis request so stale results can be dropped.
    generation: Arc<AtomicU64>,
}

fn main() {
    let arg = std::env::args().nth(1);
    if matches!(arg.as_deref(), Some("-h" | "--help")) {
        println!("usage: spectre-gui [FILE]");
        return;
    }

    let ui = MainWindow::new().expect("failed to create the window");
    let ctx = Ctx::default();
    ui.set_colorbar(colorbar_image());

    ui.on_open_file({
        let (weak, ctx) = (ui.as_weak(), ctx.clone());
        move || {
            let (weak, ctx) = (weak.clone(), ctx.clone());
            let picked = slint::spawn_local(async move {
                let extensions = audio_extensions();
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("Audio files", &extensions)
                    .pick_file()
                    .await;
                if let (Some(file), Some(ui)) = (file, weak.upgrade()) {
                    ctx.analyze(&ui, file.path().to_path_buf(), true);
                }
            });
            if let Err(e) = picked {
                eprintln!("spectre-gui: cannot open file dialog: {e}");
            }
        }
    });

    ui.on_settings_changed({
        let (weak, ctx) = (ui.as_weak(), ctx.clone());
        move || {
            let path = ctx.shared.lock().unwrap().path.clone();
            if let (Some(ui), Some(path)) = (weak.upgrade(), path) {
                ctx.analyze(&ui, path, false);
            }
        }
    });

    ui.on_range_changed({
        let (weak, ctx) = (ui.as_weak(), ctx.clone());
        move |_| {
            let spec = ctx.shared.lock().unwrap().spec.clone();
            if let (Some(ui), Some(spec)) = (weak.upgrade(), spec) {
                show_image(&ui, &spec);
            }
        }
    });

    ui.on_cursor_moved({
        let (weak, ctx) = (ui.as_weak(), ctx.clone());
        move |x, y| {
            let spec = ctx.shared.lock().unwrap().spec.clone();
            if let (Some(ui), Some(cell)) =
                (weak.upgrade(), spec.and_then(|s| s.at(x.into(), y.into())))
            {
                let freq = if cell.freq_hz >= 1000.0 {
                    format!("{:.2} kHz", cell.freq_hz / 1000.0)
                } else {
                    format!("{:.0} Hz", cell.freq_hz)
                };
                let time = axis::format_time(cell.time_secs, 0.1);
                ui.set_readout(format!("{freq}   {time}   {:.1} dBFS", cell.db).into());
            }
        }
    });

    ui.on_cursor_left({
        let weak = ui.as_weak();
        move || {
            if let Some(ui) = weak.upgrade() {
                ui.set_readout(SharedString::new());
            }
        }
    });

    if let Some(path) = arg {
        ctx.analyze(&ui, PathBuf::from(path), true);
    }
    ui.run().expect("event loop failed");
}

impl Ctx {
    /// Analyzes `path` on a worker thread and applies the result on the UI thread.
    fn analyze(&self, ui: &MainWindow, path: PathBuf, new_file: bool) {
        let params = Params {
            fft_size: FFT_SIZES[ui.get_fft_index().clamp(0, FFT_SIZES.len() as i32 - 1) as usize],
            width: COLUMNS,
            window: WINDOWS[ui.get_window_index().clamp(0, WINDOWS.len() as i32 - 1) as usize],
            // Channel 0 in the menu is the mix; a new file always starts there.
            channel: match ui.get_channel_index() {
                i if i > 0 && !new_file => Channel::Index(i as u32 - 1),
                _ => Channel::Mix,
            },
        };

        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        ui.set_busy(true);
        ui.set_error_text(SharedString::new());

        let (ctx, weak) = (self.clone(), ui.as_weak());
        std::thread::spawn(move || {
            let result = run_analysis(&path, &params, new_file);
            if ctx.generation.load(Ordering::SeqCst) != generation {
                return; // A newer request superseded this one.
            }
            // A failed event-loop call means the window is already gone.
            let _ = weak.upgrade_in_event_loop(move |ui| {
                if ctx.generation.load(Ordering::SeqCst) == generation {
                    ctx.apply(&ui, &path, result);
                }
            });
        });
    }

    fn apply(&self, ui: &MainWindow, path: &Path, result: Result<Loaded, String>) {
        ui.set_busy(false);
        let mut shared = self.shared.lock().unwrap();
        match result {
            Ok(loaded) => {
                shared.path = Some(loaded.path.clone());
                shared.spec = Some(loaded.spec.clone());
                drop(shared);

                if loaded.new_file {
                    let names: Vec<SharedString> = match loaded.channels {
                        1 => vec!["Mono".into()],
                        n => std::iter::once("Mix".into())
                            .chain((1..=n).map(|c| format!("Channel {c}").into()))
                            .collect(),
                    };
                    ui.set_channel_names(ModelRc::new(VecModel::from(names)));
                    ui.set_channel_index(0);
                }
                let name = loaded
                    .path
                    .file_name()
                    .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
                ui.set_window_title(format!("{name} — Spectre").into());
                ui.set_info_text(loaded.summary.into());
                ui.set_time_ticks(tick_model(&axis::time_ticks(loaded.spec.duration_secs, 10)));
                ui.set_freq_ticks(tick_model(&axis::frequency_ticks(
                    f64::from(loaded.spec.sample_rate) / 2.0,
                    8,
                )));
                show_image(ui, &loaded.spec);
                ui.set_has_image(true);
            }
            Err(message) => {
                // Keep the path (settings can be retried) but drop the stale picture.
                shared.path = Some(path.to_path_buf());
                shared.spec = None;
                drop(shared);
                ui.set_has_image(false);
                ui.set_info_text(SharedString::new());
                ui.set_window_title("Spectre".into());
                ui.set_error_text(message.into());
            }
        }
    }
}

/// Worker-thread half: decode through libsndfile, then compute the spectrogram.
fn run_analysis(path: &Path, params: &Params, new_file: bool) -> Result<Loaded, String> {
    let mut reader = Reader::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let info = *reader.info();
    let spec = spectrogram::analyze(&mut reader, params)
        .map_err(|e| format!("{}: {e}", path.display()))?;

    let secs = info.duration_secs();
    let summary = format!(
        "{} · {} Hz · {} ch · {}",
        info.format,
        info.sample_rate,
        info.channels,
        axis::format_time(secs, if secs < 60.0 { 0.1 } else { 1.0 })
    );
    Ok(Loaded {
        path: path.to_path_buf(),
        spec: Arc::new(spec),
        summary,
        channels: info.channels,
        new_file,
    })
}

/// Rasterizes at the current range and refreshes everything that depends on it.
fn show_image(ui: &MainWindow, spec: &Spectrogram) {
    let range_db = ui.get_range_db();
    let opts = RenderOptions {
        height: Some((spec.bins - 1).min(MAX_IMAGE_HEIGHT)),
        range_db,
    };
    let (w, h, pixels) = spectrogram::rasterize(spec, &opts);
    ui.set_spectrogram(rgb_image(&pixels, w, h));
    ui.set_level_ticks(tick_model(&axis::level_ticks(f64::from(range_db), 6)));
}

fn rgb_image(pixels: &[u8], width: usize, height: usize) -> Image {
    let mut buffer = SharedPixelBuffer::<Rgb8Pixel>::new(width as u32, height as u32);
    buffer.make_mut_bytes().copy_from_slice(pixels);
    Image::from_rgb8(buffer)
}

/// A one-pixel-wide gradient, loudest (0 dBFS) at the top, matching the spectrogram colours.
fn colorbar_image() -> Image {
    let mut pixels = Vec::with_capacity(COLORBAR_STEPS * 3);
    for i in 0..COLORBAR_STEPS {
        pixels.extend(spectrogram::colormap(
            1.0 - i as f32 / (COLORBAR_STEPS - 1) as f32,
        ));
    }
    rgb_image(&pixels, 1, COLORBAR_STEPS)
}

fn tick_model(ticks: &[Tick]) -> ModelRc<TickMark> {
    let marks: Vec<TickMark> = ticks
        .iter()
        .map(|t| TickMark {
            position: t.position as f32,
            label: t.label.as_str().into(),
        })
        .collect();
    ModelRc::new(VecModel::from(marks))
}

/// File extensions to offer in the open dialog: those libsndfile reports plus common
/// ones whose container is listed under another name.
fn audio_extensions() -> Vec<String> {
    let mut exts: Vec<String> = sndfile::formats()
        .into_iter()
        .filter_map(|f| f.info.extension)
        .collect();
    exts.extend(["aif", "mp3", "ogg", "opus", "oga", "flac", "wav"].map(String::from));
    exts.sort();
    exts.dedup();
    exts
}
