use std::f32::consts::PI;

use glam::Vec4;

pub fn triangle(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> (Vec<Vec4>, Vec<u32>) {
    (
        vec![
            Vec4::new(a.0, a.1, 0., 1.),
            Vec4::new(b.0, b.1, 0., 1.),
            Vec4::new(c.0, c.1, 0., 1.),
        ],
        vec![0, 1, 2],
    )
}

pub fn square() -> (Vec<Vec4>, Vec<u32>) {
    (
        vec![
            Vec4::new(1., 1., 0., 1.),
            Vec4::new(-1., 1., 0., 1.),
            Vec4::new(-1., -1., 0., 1.),
            Vec4::new(1., -1., 0., 1.),
        ],
        vec![0, 1, 2, 2, 3, 0],
    )
}

pub fn circle() -> (Vec<Vec4>, Vec<u32>) {
    polygon(512)
}

pub struct TriangleStrip(Vec<Vec4>, Vec<u32>);

impl TriangleStrip {
    pub fn vertex(&mut self, x: f32, y: f32) {
        self.0.push(Vec4::new(x, y, 0., 1.));

        let length = (self.0.len()) as u32 - 1;
        if length < 3 {
            self.1.push(length);
            return;
        }

        self.1.push(length - 2);
        self.1.push(length - 1);
        self.1.push(length);
    }
}

pub fn triangle_strip(call: impl FnOnce(&mut TriangleStrip)) -> (Vec<Vec4>, Vec<u32>) {
    let mut strip = TriangleStrip(vec![], vec![]);

    call(&mut strip);

    (strip.0, strip.1)
}

pub fn polygon(res: u32) -> (Vec<Vec4>, Vec<u32>) {
    let mut vertexes: Vec<Vec4> = vec![];
    let mut indexes: Vec<u32> = vec![];

    //Center vertex
    vertexes.push(Vec4::new(0., 0., 0., 1.));

    let ph = (PI * 2.) / (res as f32);
    let mut th = 0.;

    for i in 1..=res {
        let x = f32::cos(th);
        let y = f32::sin(th);

        vertexes.push(Vec4::new(x, y, 0., 1.));

        indexes.push(0);
        indexes.push(i);
        indexes.push(i + 1);

        th += ph;
    }

    indexes.pop();
    indexes.push(1);

    (vertexes, indexes)
}
