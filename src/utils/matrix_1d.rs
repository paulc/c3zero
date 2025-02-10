use anyhow::{bail, Result};

use crate::rgb::Rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

const WIDTH: usize = 8;
const HEIGHT: usize = 8;

#[derive(Clone, Debug)]
pub struct Matrix1D<const N: usize> {
    leds: [[Rgb; WIDTH * HEIGHT]; N],
    orientation: Orientation,
}

impl<const N: usize> Matrix1D<N> {
    pub fn new(orientation: Orientation) -> Self {
        let leds = [[Rgb::new(0, 0, 0); WIDTH * HEIGHT]; N];
        Self { leds, orientation }
    }
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
    pub fn clear(&mut self) {
        for i in 0..N {
            for j in 0..WIDTH * HEIGHT {
                self.leds[i][j] = Rgb::new(0, 0, 0);
            }
        }
    }
    pub fn set(&mut self, (x, y): (usize, usize), rgb: Rgb) -> Result<()> {
        if y > HEIGHT - 1 || x > (WIDTH * N) - 1 {
            bail!("Coords out of bounds: ({x},{y}");
        }
        let (i, x) = (x / WIDTH, x % WIDTH);
        self.leds[i][x + y * WIDTH] = rgb;
        Ok(())
    }
    pub fn iter(&self) -> Matrix1DIterator<'_, N> {
        Matrix1DIterator {
            leds: &self.leds,
            index: 0,
            orientation: self.orientation,
        }
    }
}

pub struct Matrix1DIterator<'a, const N: usize> {
    leds: &'a [[Rgb; WIDTH * HEIGHT]; N],
    orientation: Orientation,
    index: usize,
}

impl<const N: usize> Iterator for Matrix1DIterator<'_, N> {
    type Item = Rgb;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < WIDTH * HEIGHT * N {
            let (x, y) = (self.index % WIDTH, self.index / WIDTH);
            //let (i, x) = (x / WIDTH, x % WIDTH);
            let out = match self.orientation {
                Orientation::North => {
                    let (i, x) = (x / WIDTH, x % WIDTH);
                    self.leds[i][x + y * WIDTH]
                }
                Orientation::East => {
                    let (x1, y1) = (y, WIDTH - x - 1);
                    let (i, x1) = (x1 / WIDTH, x1 % WIDTH);
                    self.leds[i][x1 + y1 * WIDTH]
                }
                Orientation::South => {
                    let (x1, y1) = (WIDTH - x - 1, WIDTH - y - 1);
                    let (i, x1) = (x1 / WIDTH, x1 % WIDTH);
                    self.leds[i][x1 + y1 * WIDTH]
                }
                Orientation::West => {
                    let (x1, y1) = (WIDTH - y - 1, x);
                    let (i, y1) = (y1 / WIDTH, y1 % WIDTH);
                    self.leds[i][x1 + y1 * WIDTH]
                }
            };
            self.index += 1;
            Some(out)
        } else {
            None
        }
    }
}
