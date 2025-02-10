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

pub struct Matrix {
    leds: [Rgb; WIDTH * HEIGHT],
    orientation: Orientation,
}

impl Matrix {
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
    pub fn draw_glyph(&mut self, glyph: [u8; 8], colour: Rgb, offset: i8) {
        // Glyph is in MSB-LSB format (opposite to char)
        for (y, row) in glyph.into_iter().enumerate() {
            /*
            let row = match offset {
                ..-7 => 0,
                -7..0 => row << -offset,
                0 => *row,
                1..8 => row >> offset,
                8.. => 0,
            };
            */
            for x in 0..8 {
                if shift(row.reverse_bits(), offset) & (1 << x) != 0 {
                    self.leds[x + y * HEIGHT] = colour;
                }
            }
        }
    }

    pub fn draw_char(&mut self, c: char, colour: Rgb, offset: i8) {
        if let Some(glyph) = BASIC_FONTS.get(c) {
            for (y, row) in glyph.into_iter().enumerate() {
                /*
                let row = match offset {
                    ..-7 => 0,
                    -7..0 => row >> -offset,
                    0 => *row,
                    1..8 => row << offset,
                    8.. => 0,
                };
                */
                for x in 0..8 {
                    if shift(row, offset) & (1 << x) != 0 {
                        self.leds[x + y * HEIGHT] = colour;
                    }
                }
            }
        }
    }
    pub fn draw_bitmap(&mut self, bitmap: &[&str; 8], colourmap: &[(char, Rgb)], _offset: i8) {
        // Glyph is in MSB-LSB format (opposite to char)
        for (y, &row) in bitmap.iter().enumerate() {
            for (x, c) in row.chars().take(8).enumerate() {
                let mut colour = Rgb::new(0, 0, 0);
                for (key, rgb) in colourmap {
                    if c == *key {
                        colour = *rgb;
                        break;
                    }
                }
                self.leds[x + y * HEIGHT] = colour;
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
    pub fn iter(&self) -> MatrixIterator {
        MatrixIterator {
            leds: &self.leds,
            index: 0,
            orientation: self.orientation,
        }
    }
}

pub struct MatrixIterator<'a> {
    leds: &'a [Rgb; WIDTH * HEIGHT],
    orientation: Orientation,
    index: usize,
}

impl Iterator for MatrixIterator<'_> {
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

// Handle +/- shift & offset > width
fn shift(value: u8, offset: i8) -> u8 {
    match offset {
        ..-7 => 0,
        -7..0 => value >> -offset,
        0 => value,
        1..8 => value << offset,
        8.. => 0,
    }
}
