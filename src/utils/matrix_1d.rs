use anyhow::{bail, Result};

use crate::rgb::{Rgb, OFF};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation::North
    }
}

const WIDTH: usize = 8;
const HEIGHT: usize = 8;
const PANEL_PIXELS: usize = WIDTH * HEIGHT;

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
    pub fn set(&mut self, (x, y): (usize, usize), rgb: Rgb) -> Result<()> {
        if y > HEIGHT - 1 || x > (WIDTH * N) - 1 {
            bail!("Coords out of bounds: ({x},{y}");
        }
        let (i, x) = (x / WIDTH, x % WIDTH);
        self.panels[i].leds[x + y * WIDTH] = rgb;
        Ok(())
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
