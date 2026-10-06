use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use spectre::dsp::Window;
use spectre::sndfile::{self, Reader};
use spectre::spectrogram::{self, Channel, Params, RenderOptions};

#[derive(Parser)]
#[command(
    name = "spectre",
    version,
    about = "Acoustic spectrum analyzer built on libsndfile"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Show stream parameters and tags of an audio file.
    Info { file: PathBuf },

    /// Render a spectrogram of an audio file to a PNG image.
    Render {
        file: PathBuf,

        /// Output image path [default: <file>.png].
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// FFT size in samples (power of two).
        #[arg(short = 'n', long, default_value_t = 2048)]
        fft_size: usize,

        /// Image width in pixels (one column per time slice).
        #[arg(short, long, default_value_t = 1024)]
        width: usize,

        /// Image height in pixels [default: fft-size / 2].
        #[arg(short = 'H', long)]
        height: Option<usize>,

        /// Window function: rectangular, hann, hamming or blackman.
        #[arg(long, default_value = "hann")]
        window: Window,

        /// Dynamic range in dB below full scale.
        #[arg(short, long, default_value_t = 120.0)]
        range: f32,

        /// Analyze one zero-based channel instead of the mono mix.
        #[arg(short, long)]
        channel: Option<u32>,
    },

    /// List the container formats and codecs this libsndfile build supports.
    Formats,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("spectre: {e}");
            ExitCode::FAILURE
        }
    }
}

type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

fn run(cli: Cli) -> BoxResult<()> {
    match cli.command {
        Cmd::Info { file } => info(&file),
        Cmd::Formats => formats(),
        Cmd::Render {
            file,
            output,
            fft_size,
            width,
            height,
            window,
            range,
            channel,
        } => {
            let output = output.unwrap_or_else(|| {
                let mut name = file.as_os_str().to_owned();
                name.push(".png");
                PathBuf::from(name)
            });
            let mut reader = open(&file)?;
            let params = Params {
                fft_size,
                width,
                window,
                channel: channel.map_or(Channel::Mix, Channel::Index),
            };
            let spec = spectrogram::analyze(&mut reader, &params)?;
            let opts = RenderOptions {
                height,
                range_db: range,
            };
            spectrogram::write_png(&spec, &opts, &output)?;
            println!(
                "wrote {} ({}x{})",
                output.display(),
                spec.columns,
                height.unwrap_or(spec.bins - 1)
            );
            if let Some((hz, db)) = spec.peak() {
                println!("peak: {hz:.1} Hz at {db:.1} dBFS");
            }
            Ok(())
        }
    }
}

fn open(file: &PathBuf) -> BoxResult<Reader> {
    Reader::open(file).map_err(|e| format!("{}: {e}", file.display()).into())
}

fn info(file: &PathBuf) -> BoxResult<()> {
    let reader = open(file)?;
    let i = reader.info();
    println!("File:        {}", file.display());
    println!("Format:      {}", i.format);
    println!("Sample rate: {} Hz", i.sample_rate);
    println!("Channels:    {}", i.channels);
    if i.frames > 0 {
        let secs = i.duration_secs();
        println!(
            "Duration:    {}:{:06.3} ({} frames)",
            (secs / 60.0) as u64,
            secs % 60.0,
            i.frames
        );
    } else {
        println!("Duration:    unknown");
    }
    let tags = reader.tags();
    for (label, value) in [
        ("Title", &tags.title),
        ("Artist", &tags.artist),
        ("Album", &tags.album),
        ("Date", &tags.date),
        ("Comment", &tags.comment),
        ("Software", &tags.software),
        ("Copyright", &tags.copyright),
    ] {
        if let Some(v) = value {
            println!("{:<12} {v}", format!("{label}:"));
        }
    }
    Ok(())
}

fn formats() -> BoxResult<()> {
    println!("{}", sndfile::lib_version());
    for f in sndfile::formats() {
        let ext = f.info.extension.as_deref().unwrap_or("-");
        println!("\n{} (.{ext})", f.info.name);
        for s in &f.subtypes {
            println!("    {}", s.name);
        }
    }
    Ok(())
}
