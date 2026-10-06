//! Window functions used before the FFT.

use std::f32::consts::PI;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    Rectangular,
    Hann,
    Hamming,
    Blackman,
}

impl Window {
    /// Symmetric window coefficients of length `n`.
    pub fn coefficients(self, n: usize) -> Vec<f32> {
        if n <= 1 {
            return vec![1.0; n];
        }
        let denom = (n - 1) as f32;
        (0..n)
            .map(|i| {
                let x = 2.0 * PI * i as f32 / denom;
                match self {
                    Window::Rectangular => 1.0,
                    Window::Hann => 0.5 - 0.5 * x.cos(),
                    Window::Hamming => 0.54 - 0.46 * x.cos(),
                    Window::Blackman => 0.42 - 0.5 * x.cos() + 0.08 * (2.0 * x).cos(),
                }
            })
            .collect()
    }
}

impl FromStr for Window {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "rectangular" | "rect" | "none" => Ok(Window::Rectangular),
            "hann" | "hanning" => Ok(Window::Hann),
            "hamming" => Ok(Window::Hamming),
            "blackman" => Ok(Window::Blackman),
            _ => Err(format!(
                "unknown window '{s}' (expected rectangular, hann, hamming or blackman)"
            )),
        }
    }
}

impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Window::Rectangular => "rectangular",
            Window::Hann => "hann",
            Window::Hamming => "hamming",
            Window::Blackman => "blackman",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hann_is_zero_at_edges_and_one_in_the_middle() {
        let w = Window::Hann.coefficients(65);
        assert!(w[0].abs() < 1e-6 && w[64].abs() < 1e-6);
        assert!((w[32] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn windows_are_symmetric() {
        for win in [Window::Hann, Window::Hamming, Window::Blackman] {
            let w = win.coefficients(128);
            for i in 0..64 {
                assert!(
                    (w[i] - w[127 - i]).abs() < 1e-5,
                    "{win} not symmetric at {i}"
                );
            }
        }
    }

    #[test]
    fn parses_names() {
        assert_eq!("Hann".parse::<Window>().unwrap(), Window::Hann);
        assert!("bogus".parse::<Window>().is_err());
    }
}
