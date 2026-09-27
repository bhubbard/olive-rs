use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RgbaColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RgbaColor {
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const RED: Self = Self { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Self = Self { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Self = Self { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn clamp(&self) -> Self {
        Self {
            r: self.r.clamp(0.0, 1.0),
            g: self.g.clamp(0.0, 1.0),
            b: self.b.clamp(0.0, 1.0),
            a: self.a.clamp(0.0, 1.0),
        }
    }

    pub fn to_u8_array(&self) -> [u8; 4] {
        let c = self.clamp();
        [
            (c.r * 255.0).round() as u8,
            (c.g * 255.0).round() as u8,
            (c.b * 255.0).round() as u8,
            (c.a * 255.0).round() as u8,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>, // RGBA float values, size = width * height * 4
}

impl RgbaImage {
    pub fn new(width: u32, height: u32, fill: RgbaColor) -> Self {
        let pixel_count = (width * height) as usize;
        let mut data = Vec::with_capacity(pixel_count * 4);
        for _ in 0..pixel_count {
            data.push(fill.r);
            data.push(fill.g);
            data.push(fill.b);
            data.push(fill.a);
        }
        Self { width, height, data }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> RgbaColor {
        if x >= self.width || y >= self.height {
            return RgbaColor::TRANSPARENT;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        RgbaColor {
            r: self.data[idx],
            g: self.data[idx + 1],
            b: self.data[idx + 2],
            a: self.data[idx + 3],
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: RgbaColor) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        self.data[idx] = color.r;
        self.data[idx + 1] = color.g;
        self.data[idx + 2] = color.b;
        self.data[idx + 3] = color.a;
    }

    pub fn to_rgba8_vec(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.data.len());
        for chunk in self.data.chunks_exact(4) {
            let r = (chunk[0].clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = (chunk[1].clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = (chunk[2].clamp(0.0, 1.0) * 255.0).round() as u8;
            let a = (chunk[3].clamp(0.0, 1.0) * 255.0).round() as u8;
            out.push(r);
            out.push(g);
            out.push(b);
            out.push(a);
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioBuffer {
    pub channels: u32,
    pub sample_rate: u32,
    pub samples: Vec<f32>, // interleaved audio samples
}

impl AudioBuffer {
    pub fn new(channels: u32, sample_rate: u32, sample_count_per_channel: usize) -> Self {
        Self {
            channels,
            sample_rate,
            samples: vec![0.0; sample_count_per_channel * channels as usize],
        }
    }

    pub fn silence(channels: u32, sample_rate: u32, duration_secs: f32) -> Self {
        let count = (duration_secs * sample_rate as f32).round() as usize;
        Self::new(channels, sample_rate, count)
    }

    pub fn mix(&mut self, other: &AudioBuffer, volume: f32) {
        let len = self.samples.len().min(other.samples.len());
        for i in 0..len {
            self.samples[i] = (self.samples[i] + other.samples[i] * volume).clamp(-1.0, 1.0);
        }
    }
}
