use anyhow::{bail, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RgbLayout {
    Rgb,
    Grb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

pub const OFF: Rgb = Rgb { r: 0, g: 0, b: 0 };
pub const RED: Rgb = Rgb { r: 255, g: 0, b: 0 };
pub const GREEN: Rgb = Rgb { r: 0, g: 255, b: 0 };
pub const BLUE: Rgb = Rgb { r: 0, g: 0, b: 255 };
pub const WHITE: Rgb = Rgb {
    r: 255,
    g: 255,
    b: 255,
};

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    /// Converts hue, saturation, value to RGB
    pub fn from_hsv(h: u32, s: u32, v: u32) -> Result<Self> {
        if h > 360 || s > 100 || v > 100 {
            bail!("The given HSV values are not in valid range");
        }
        let s = s as f64 / 100.0;
        let v = v as f64 / 100.0;
        let c = s * v;
        let x = c * (1.0 - (((h as f64 / 60.0) % 2.0) - 1.0).abs());
        let m = v - c;
        let (r, g, b) = match h {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        Ok(Self {
            r: ((r + m) * 255.0) as u8,
            g: ((g + m) * 255.0) as u8,
            b: ((b + m) * 255.0) as u8,
        })
    }
    pub fn to_u32(&self, format: RgbLayout) -> u32 {
        match format {
            RgbLayout::Rgb => ((self.r as u32) << 16) | ((self.g as u32) << 8) | self.b as u32,
            RgbLayout::Grb => ((self.g as u32) << 16) | ((self.r as u32) << 8) | self.b as u32,
        }
    }
}
