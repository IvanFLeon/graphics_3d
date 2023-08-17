use glam::Vec4;

#[derive(Clone, Copy, Default)]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Color { r, g, b, a }
    }
}

impl From<[f64; 4]> for Color {
    fn from(value: [f64; 4]) -> Self {
        Self::new(value[0], value[1], value[2], value[3])
    }
}

impl From<Color> for Vec4 {
    fn from(value: Color) -> Self {
        Self::new(
            value.r as f32,
            value.g as f32,
            value.b as f32,
            value.a as f32,
        )
    }
}

impl From<Color> for wgpu::Color {
    fn from(c: Color) -> Self {
        wgpu::Color {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}
