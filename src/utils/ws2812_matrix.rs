use font8x8::{UnicodeFonts, BASIC_FONTS};

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

pub struct Ws2812Matrix {
    leds: [Rgb; WIDTH * HEIGHT],
    orientation: Orientation,
}

impl Ws2812Matrix {
    pub fn new(orientation: Orientation) -> Self {
        Self {
            leds: [Rgb::new(0, 0, 0); WIDTH * HEIGHT],
            orientation,
        }
    }
    pub fn fill(&mut self, c: Rgb) {
        for i in 0..(WIDTH * HEIGHT) {
            self.leds[i] = c;
        }
    }
    pub fn draw_char(&mut self, c: char, colour: Rgb, offset: i8) {
        if let Some(glyph) = BASIC_FONTS.get(c) {
            for (y, row) in glyph.iter().enumerate() {
                for x in 0..8 {
                    if row & (1 << x) != 0 {
                        let x1 = x as i8 + offset;
                        if (0..WIDTH as i8).contains(&x1) {
                            self.leds[(x1 as usize) + y * HEIGHT] = colour;
                        }
                    }
                }
            }
        }
    }
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
    pub fn set(&mut self, xy: (usize, usize), c: Rgb) {
        self.leds[xy.0 + xy.1 * HEIGHT] = c;
    }
    pub fn get(&mut self, xy: (usize, usize)) -> Rgb {
        self.leds[xy.0 + xy.1 * HEIGHT]
    }
    pub fn iter(&self) -> Ws2812MatrixIterator {
        Ws2812MatrixIterator {
            leds: &self.leds,
            index: 0,
            orientation: self.orientation,
        }
    }
}

pub struct Ws2812MatrixIterator<'a> {
    leds: &'a [Rgb; WIDTH * HEIGHT],
    orientation: Orientation,
    index: usize,
}

impl Iterator for Ws2812MatrixIterator<'_> {
    type Item = Rgb;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < WIDTH * HEIGHT {
            let out = match self.orientation {
                Orientation::North => self.leds[self.index],
                Orientation::East => {
                    let (x, y) = (self.index % WIDTH, self.index / WIDTH);
                    let (x1, y1) = (y, WIDTH - x - 1);
                    self.leds[x1 + (HEIGHT * y1)]
                }
                Orientation::South => {
                    let (x, y) = (self.index % WIDTH, self.index / WIDTH);
                    let (x1, y1) = (WIDTH - x - 1, WIDTH - y - 1);
                    self.leds[x1 + (HEIGHT * y1)]
                }
                Orientation::West => {
                    let (x, y) = (self.index % WIDTH, self.index / WIDTH);
                    let (x1, y1) = (WIDTH - y - 1, x);
                    self.leds[x1 + (HEIGHT * y1)]
                }
            };
            self.index += 1;
            Some(out)
        } else {
            None
        }
    }
}
