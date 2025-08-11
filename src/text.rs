use std::ops::{Add, Div, Mul, Sub};

use crate::math::vectors::Vec2;
use serde::{Deserialize, Serialize};

pub static ATLAS: std::sync::LazyLock<image::RgbaImage> = std::sync::LazyLock::new(|| {
    image::load_from_memory(include_bytes!("atlas.png"))
        .unwrap()
        .to_rgba8()
});

static ATLAS_DATA: std::sync::LazyLock<std::collections::HashMap<char, CharData>> =
    std::sync::LazyLock::new(|| {
        toml::from_str::<std::collections::HashMap<char, Sprite>>(include_str!("atlas.toml"))
            .unwrap()
            .iter()
            .map(|(char, sprite)| (*char, sprite.into()))
            .collect()
    });

struct AtlasData<T = f32> {
    width: T,
    height: T,
    char_data: std::collections::HashMap<char, CharData>,
}

struct CharData<T = f32> {
    tex_rect: TexRect2D<T>,
    advance: T,
}

#[derive(Debug, Copy, Clone)]
struct TexRect2D<T = f32> {
    start: Vec2<T>,
    end: Vec2<T>,
    start_uv: Vec2<T>,
    end_uv: Vec2<T>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct Size {
    width: u32,
    height: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct Rectangle {
    x: u32,
    y: u32,
    #[serde(alias = "sizeX")]
    width: u32,
    #[serde(alias = "sizeY")]
    height: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct Character {
    value: u32,
    offset: Position,
    advance_x: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Sprite {
    name_id: String,
    tag: String,
    origin: Position,
    position: Position,
    source_size: Size,
    padding: u32,
    trimmed: bool,
    trim_rec: Rectangle,
    collider_type: u32,
    collider_info: Rectangle,
    char: Character,
}

impl From<&Sprite> for CharData {
    fn from(value: &Sprite) -> Self {
        let scale = Vec2::new(ATLAS.width() as f32, ATLAS.height() as f32);

        Self {
            tex_rect: TexRect2D {
                start: Vec2::new(
                    value.trim_rec.x as f32 + value.char.offset.x as f32,
                    -(value.trim_rec.height as f32 + value.char.offset.y as f32),
                ) / scale,
                end: Vec2::new(
                    value.trim_rec.width as f32 + value.char.offset.x as f32,
                    -(value.trim_rec.y as f32 + value.char.offset.y as f32),
                ) / scale,
                start_uv: Vec2::new(
                    (value.position.x) as f32,
                    (value.source_size.height as i32 + value.position.y) as f32,
                ) / scale,
                end_uv: Vec2::new(
                    (value.source_size.width as i32 + value.position.x) as f32,
                    (value.position.y) as f32,
                ) / scale,
            },

            advance: value.char.advance_x as f32 / scale.x,
        }
    }
}

impl<T: Add<Output = T> + Copy> Add<Vec2<T>> for TexRect2D<T> {
    type Output = TexRect2D<T>;

    fn add(self, rhs: Vec2<T>) -> Self::Output {
        TexRect2D {
            start: self.start + rhs,
            end: self.end + rhs,
            start_uv: self.start_uv,
            end_uv: self.end_uv,
        }
    }
}

impl<T: Sub<Output = T> + Copy> Sub<Vec2<T>> for TexRect2D<T> {
    type Output = TexRect2D<T>;

    fn sub(self, rhs: Vec2<T>) -> Self::Output {
        TexRect2D {
            start: self.start - rhs,
            end: self.end - rhs,
            start_uv: self.start_uv,
            end_uv: self.end_uv,
        }
    }
}

impl<T: Mul<Output = T> + Copy> Mul<T> for TexRect2D<T> {
    type Output = TexRect2D<T>;

    fn mul(self, rhs: T) -> Self {
        TexRect2D {
            start: self.start * rhs,
            end: self.end * rhs,
            start_uv: self.start_uv,
            end_uv: self.end_uv,
        }
    }
}

impl<T: Div<Output = T> + Copy> Div<T> for TexRect2D<T> {
    type Output = TexRect2D<T>;

    fn div(self, rhs: T) -> Self {
        TexRect2D {
            start: self.start / rhs,
            end: self.end / rhs,
            start_uv: self.start_uv,
            end_uv: self.end_uv,
        }
    }
}

pub struct Line<T = f32> {
    tex_rects: Vec<TexRect2D<T>>,
    advance: T,
}

impl Line {
    pub fn new(text: &str) -> Self {
        let mut line = Line {
            tex_rects: Vec::new(),
            advance: 0.0,
        };

        line.set(text);

        line
    }

    pub fn set(&mut self, text: &str) {
        let atlas = &ATLAS_DATA;

        self.advance = 0.0;

        self.tex_rects = text
            .chars()
            .filter_map(|c| {
                let data = atlas.get(&c);
                println!("{c}:{}", data.unwrap().advance);
                data
            })
            .map(|data| {
                let rect = data.tex_rect + Vec2::new(self.advance, 0.0);

                self.advance += data.advance;

                rect
            })
            .collect();
    }

    pub fn push(&mut self, text: &str) {
        let atlas = &ATLAS_DATA;

        let mut new = text
            .chars()
            .filter_map(|c| {
                let data = atlas.get(&c);
                println!("{c}:{}", data.unwrap().advance);
                data
            })
            .map(|data| {
                let rect = data.tex_rect + Vec2::new(self.advance, 0.0);

                self.advance += data.advance;

                rect
            })
            .collect();

        self.tex_rects.append(&mut new);
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { crate::util::as_u8_slice_from_slice(&self.tex_rects) }
    }
}
