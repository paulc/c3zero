use font8x8::{UnicodeFonts, BASIC_FONTS};
use std::iter::Rev;
use std::ops::Range;

use crate::rgb::{Rgb, RgbTransform, OFF};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Orientation {
    #[default]
    North,
    East,
    South,
    West,
}

pub const WIDTH: usize = 8;
pub const HEIGHT: usize = 8;
pub const PANEL_PIXELS: usize = WIDTH * HEIGHT;
pub const CHAR_WIDTH: usize = 8;

#[derive(Clone, Debug)]
pub struct Panel {
    leds: [Rgb; WIDTH * HEIGHT],
    orientation: Orientation,
}

impl Panel {
    pub fn new(orientation: Orientation) -> Self {
        let leds = [Rgb::new(0, 0, 0); PANEL_PIXELS];
        Self { leds, orientation }
    }
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
    pub fn clear(&mut self) {
        for j in 0..PANEL_PIXELS {
            self.leds[j] = OFF;
        }
    }
    pub fn iter(&self) -> PanelIterator<'_> {
        PanelIterator {
            panel: self,
            index: 0,
        }
    }
}

pub struct PanelIterator<'a> {
    panel: &'a Panel,
    index: usize,
}

impl Iterator for PanelIterator<'_> {
    type Item = Rgb;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < PANEL_PIXELS {
            let (x, y) = (self.index % WIDTH, self.index / WIDTH);
            let out = match self.panel.orientation {
                Orientation::North => self.panel.leds[x + y * WIDTH],
                Orientation::East => {
                    let (x1, y1) = (y, WIDTH - x - 1);
                    self.panel.leds[x1 + y1 * WIDTH]
                }
                Orientation::South => {
                    let (x1, y1) = (WIDTH - x - 1, WIDTH - y - 1);
                    self.panel.leds[x1 + y1 * WIDTH]
                }
                Orientation::West => {
                    let (x1, y1) = (WIDTH - y - 1, x);
                    self.panel.leds[x1 + y1 * WIDTH]
                }
            };
            self.index += 1;
            Some(out)
        } else {
            None
        }
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            leds: [Rgb::default(); PANEL_PIXELS],
            orientation: Orientation::North,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Matrix1D<const N: usize> {
    panels: [Panel; N],
}

impl<const N: usize> Default for Matrix1D<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Matrix1D<N> {
    pub fn new() -> Self {
        let panels = std::array::from_fn(|_| Panel::default());
        Self { panels }
    }
    pub fn from_panels(panels: [Panel; N]) -> Self {
        Self { panels }
    }
    pub fn clear(&mut self) {
        (0..N).for_each(|i| self.panels[i].clear())
    }
    // Pass (x,y) as i32 to handle transformations more easily
    pub fn set(&mut self, (x, y): (i32, i32), rgb: Rgb) {
        if (0..HEIGHT as i32).contains(&y) && (0..(N * WIDTH) as i32).contains(&x) {
            let (x, y) = (x as usize, y as usize);
            let (i, x) = (x / WIDTH, x % WIDTH);
            self.panels[i].leds[x + y * WIDTH] = rgb;
        }
    }
    pub fn transform(&mut self, (x1, y1): (i32, i32), (x2, y2): (i32, i32), t: &[RgbTransform]) {
        for x in x1..x2 {
            for y in y1..y2 {
                if (0..HEIGHT as i32).contains(&y) && (0..(N * WIDTH) as i32).contains(&x) {
                    let (x, y) = (x as usize, y as usize);
                    let (i, x) = (x / WIDTH, x % WIDTH);
                    let rgb = self.panels[i].leds[x + y * WIDTH];
                    self.panels[i].leds[x + y * WIDTH] = rgb.transform(t);
                }
            }
        }
    }
    pub fn draw_char(&mut self, c: char, rgb: Rgb, (x1, y1): (i32, i32)) {
        if let Some(glyph) = BASIC_FONTS.get(c) {
            for (y, row) in glyph.into_iter().enumerate() {
                for x in 0..8 {
                    if row & (1 << x) != 0 {
                        self.set((x1 + x, y1 + y as i32), rgb)
                    }
                }
            }
        }
    }
    pub fn draw_str(&mut self, s: &str, rgb: Rgb, (x1, y1): (i32, i32)) {
        for (i, c) in s.chars().enumerate() {
            if let Some(glyph) = BASIC_FONTS.get(c) {
                for (y, row) in glyph.into_iter().enumerate() {
                    for x in 0..8 {
                        if row & (1 << x) != 0 {
                            self.set((x1 + x + (i * CHAR_WIDTH) as i32, y1 + y as i32), rgb)
                        }
                    }
                }
            }
        }
    }
    // Returns iterator with x co-ordinates to scroll string of length len
    pub fn scroll_iter(&self, len: usize) -> Rev<Range<i32>> {
        let width = (len * CHAR_WIDTH) as i32;
        (-width..(N * WIDTH) as i32).rev()
    }
    pub fn iter(&mut self) -> Matrix1DIterator<'_, N> {
        Matrix1DIterator {
            panels: &mut self.panels,
            index: 0,
        }
    }
}

pub struct Matrix1DIterator<'a, const N: usize> {
    panels: &'a mut [Panel; N],
    index: usize,
}

impl<const N: usize> Iterator for Matrix1DIterator<'_, N> {
    type Item = Rgb;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < PANEL_PIXELS * N {
            let (panel, panel_index) = (
                &mut self.panels[self.index / PANEL_PIXELS],
                self.index % PANEL_PIXELS,
            );
            let (x, y) = (panel_index % WIDTH, panel_index / WIDTH);
            let out = match panel.orientation {
                Orientation::North => panel.leds[x + y * WIDTH],
                Orientation::East => {
                    let (x1, y1) = (y, WIDTH - x - 1);
                    panel.leds[x1 + y1 * WIDTH]
                }
                Orientation::South => {
                    let (x1, y1) = (WIDTH - x - 1, WIDTH - y - 1);
                    panel.leds[x1 + y1 * WIDTH]
                }
                Orientation::West => {
                    let (x1, y1) = (WIDTH - y - 1, x);
                    panel.leds[x1 + y1 * WIDTH]
                }
            };
            self.index += 1;
            Some(out)
        } else {
            None
        }
    }
}
