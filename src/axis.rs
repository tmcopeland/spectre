//! Axis tick generation and label formatting for spectrogram displays.
//!
//! Positions are fractions in `0.0..=1.0` measured from the axis origin (left for
//! time, bottom for frequency and level), so any front end can scale them to pixels.

#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    pub position: f64,
    pub label: String,
}

/// Smallest "nice" step (1, 2 or 5 times a power of ten) that yields at most
/// `max_ticks` intervals over `range`.
pub fn nice_step(range: f64, max_ticks: usize) -> f64 {
    assert!(range > 0.0 && max_ticks > 0);
    let raw = range / max_ticks as f64;
    let magnitude = 10f64.powf(raw.log10().floor());
    [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|m| m * magnitude)
        .find(|step| *step >= raw * (1.0 - 1e-9))
        .expect("10 * magnitude always covers raw")
}

/// Formats a number with at most two decimals, trimming trailing zeros.
fn trim(value: f64) -> String {
    let s = format!("{value:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Frequency ticks from 0 Hz to `nyquist_hz`, labelled in kHz.
pub fn frequency_ticks(nyquist_hz: f64, max_ticks: usize) -> Vec<Tick> {
    if nyquist_hz <= 0.0 {
        return Vec::new();
    }
    let step = nice_step(nyquist_hz, max_ticks);
    (0..)
        .map(|i| i as f64 * step)
        .take_while(|hz| *hz <= nyquist_hz * (1.0 + 1e-9))
        .map(|hz| Tick {
            position: (hz / nyquist_hz).min(1.0),
            label: trim(hz / 1000.0),
        })
        .collect()
}

/// Steps (in seconds) that read naturally on a clock: tenths, seconds, minutes, hours.
const TIME_STEPS: [f64; 19] = [
    0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0,
    1800.0, 3600.0, 7200.0,
];

/// Time ticks from 0 to `duration_secs`.
pub fn time_ticks(duration_secs: f64, max_ticks: usize) -> Vec<Tick> {
    if duration_secs <= 0.0 {
        return Vec::new();
    }
    let raw = duration_secs / max_ticks as f64;
    let step = TIME_STEPS
        .iter()
        .copied()
        .find(|s| *s >= raw)
        .unwrap_or_else(|| nice_step(duration_secs, max_ticks));
    (0..)
        .map(|i| i as f64 * step)
        .take_while(|t| *t <= duration_secs * (1.0 + 1e-9))
        .map(|t| Tick {
            position: (t / duration_secs).min(1.0),
            label: format_time(t, step),
        })
        .collect()
}

/// `m:ss`, `h:mm:ss`, or with fractional seconds when `step` is below one second.
pub fn format_time(secs: f64, step: f64) -> String {
    let decimals = if step >= 1.0 {
        0
    } else if step >= 0.1 {
        1
    } else {
        2
    };
    // Round once so 59.96 s cannot print as "0:60.0".
    let scale = 10f64.powi(decimals);
    let total = (secs * scale).round() / scale;
    let whole = total.trunc() as u64;
    let frac = total - whole as f64;
    let (h, m, s) = (whole / 3600, whole / 60 % 60, whole % 60);
    let secs_str = if decimals == 0 {
        format!("{s:02}")
    } else {
        format!(
            "{:0width$.dec$}",
            s as f64 + frac,
            width = 3 + decimals as usize,
            dec = decimals as usize
        )
    };
    if h > 0 {
        format!("{h}:{m:02}:{secs_str}")
    } else {
        format!("{m}:{secs_str}")
    }
}

/// Level ticks for a colour bar: 0 dBFS at the top (position 1.0) down to `-range_db`.
pub fn level_ticks(range_db: f64, max_ticks: usize) -> Vec<Tick> {
    if range_db <= 0.0 {
        return Vec::new();
    }
    let step = nice_step(range_db, max_ticks);
    (0..)
        .map(|i| i as f64 * step)
        .take_while(|d| *d <= range_db * (1.0 + 1e-9))
        .map(|d| Tick {
            position: 1.0 - (d / range_db).min(1.0),
            label: if d == 0.0 {
                "0".into()
            } else {
                format!("-{}", trim(d))
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(ticks: &[Tick]) -> Vec<&str> {
        ticks.iter().map(|t| t.label.as_str()).collect()
    }

    #[test]
    fn nice_steps_are_one_two_five() {
        assert_eq!(nice_step(120.0, 6), 20.0);
        assert_eq!(nice_step(22050.0, 8), 5000.0);
        assert_eq!(nice_step(1.0, 10), 0.1);
        assert_eq!(nice_step(7.0, 10), 1.0);
    }

    #[test]
    fn frequency_ticks_cover_the_axis_in_khz() {
        let t = frequency_ticks(22050.0, 8);
        assert_eq!(labels(&t), ["0", "5", "10", "15", "20"]);
        assert_eq!(t[0].position, 0.0);
        assert!((t[4].position - 20000.0 / 22050.0).abs() < 1e-12);

        let low = frequency_ticks(4000.0, 8);
        assert_eq!(
            labels(&low),
            ["0", "0.5", "1", "1.5", "2", "2.5", "3", "3.5", "4"]
        );
        assert_eq!(low.last().unwrap().position, 1.0);
    }

    #[test]
    fn time_labels_switch_format_with_scale() {
        assert_eq!(
            labels(&time_ticks(200.0, 10)),
            ["0:00", "0:30", "1:00", "1:30", "2:00", "2:30", "3:00"]
        );
        assert_eq!(
            labels(&time_ticks(3.0, 3)),
            ["0:00", "0:01", "0:02", "0:03"]
        );
        assert_eq!(
            labels(&time_ticks(0.5, 5)),
            ["0:00.0", "0:00.1", "0:00.2", "0:00.3", "0:00.4", "0:00.5"]
        );
        assert_eq!(
            labels(&time_ticks(7200.0, 4))[..3],
            ["0:00", "30:00", "1:00:00"]
        );
    }

    #[test]
    fn time_formatting_rounds_without_overflowing_the_seconds() {
        assert_eq!(format_time(59.96, 0.1), "1:00.0");
        assert_eq!(format_time(61.5, 1.0), "1:02"); // rounds half away from zero
        assert_eq!(format_time(3725.0, 1.0), "1:02:05");
        assert_eq!(format_time(5.25, 0.01), "0:05.25");
    }

    #[test]
    fn level_ticks_run_from_zero_at_the_top() {
        let t = level_ticks(120.0, 6);
        assert_eq!(
            labels(&t),
            ["0", "-20", "-40", "-60", "-80", "-100", "-120"]
        );
        assert_eq!(t[0].position, 1.0);
        assert_eq!(t[6].position, 0.0);
    }

    #[test]
    fn degenerate_ranges_give_no_ticks() {
        assert!(frequency_ticks(0.0, 8).is_empty());
        assert!(time_ticks(0.0, 8).is_empty());
        assert!(level_ticks(0.0, 6).is_empty());
    }
}
