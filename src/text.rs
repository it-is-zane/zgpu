use crate::math::vector::Vec2;
use serde::{Deserialize, Serialize};
use std::ops::Add;

pub static ATLAS: std::sync::LazyLock<image::RgbaImage> = std::sync::LazyLock::new(|| {
    image::load_from_memory(include_bytes!("atlas.png"))
        .unwrap()
        .to_rgba8()
});

static ATLAS_DATA: std::sync::LazyLock<AtlasData> =
    std::sync::LazyLock::new(|| toml::from_str(include_str!("atlas.toml")).unwrap());

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AtlasData<T = f32> {
    size: Vec2<T>,
    font_size: T,
    char_data: std::collections::HashMap<String, CharData>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
struct CharData<T = f32> {
    tex_rect: TexRect2D<T>,
    advance: T,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
struct TexRect2D<T = f32> {
    start: Vec2<T>,
    end: Vec2<T>,
    start_uv: Vec2<T>,
    end_uv: Vec2<T>,
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
                let data = atlas.char_data.get(&c.to_string());
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
                let data = atlas.char_data.get(&c.to_string());
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
