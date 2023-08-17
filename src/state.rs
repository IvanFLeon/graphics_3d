use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use glam::{Mat4, Quat, Vec3, Vec4};
use wgpu::util::DrawIndexedIndirect;

use crate::{
    color::Color,
    renderer::{Instance, RenderState},
};

#[derive(Default)]
pub struct State {
    pub shapes: Vec<Shape>,
    pub nodes: Vec<Node>,
    pub root: Rc<RefCell<Node>>,
    pub camera: Camera,
    pub clear: Color,
}

impl State {
    pub fn serialize(&mut self) -> RenderState {
        let mut node_stack = vec![self.root.clone()];
        let mut transform_index_stack: Vec<usize> = vec![];

        let mut vertexes: Vec<Vec4> = Vec::new();
        let mut indexes: Vec<u32> = Vec::new();
        let mut instances: Vec<Instance> = vec![Instance::default()];
        let mut draws: Vec<DrawIndexedIndirect> = vec![DrawIndexedIndirect::default()];

        loop {
            let curr_transform_index = transform_index_stack.pop();
            let Some(curr) = node_stack.pop() else {
                break;
            };
            let curr = curr.borrow_mut();

            let mut instance = Instance::default();

            if let Some(transform) = &curr.transform {
                if let Some(scale) = transform.scale {
                    instance.transform = instance.transform.mul_mat4(&Mat4::from_scale(scale));
                };

                if let Some(rotation) = transform.rotation {
                    instance.transform = instance.transform.mul_mat4(&Mat4::from_quat(rotation));
                };

                if let Some(translation) = transform.translation {
                    instance.transform = instance
                        .transform
                        .mul_mat4(&Mat4::from_translation(translation));
                };

                if let Some(i) = curr_transform_index {
                    instance.transform = instances[i].transform.mul_mat4(&instance.transform);
                };
            }

            instance.color = match curr.color {
                Some(color) => color.into(),
                None => match curr_transform_index {
                    Some(i) => instances[i].color,
                    None => instance.color,
                },
            };

            instances.push(instance);

            for i in &curr.shapes {
                let shape = &self.shapes[*i];

                let (mut vx, mut ix) = match shape {
                    Shape::Triangle(a, b, c) => {
                        let a = Vec4::from((a[0], a[1], a[2], 1.));
                        let b = Vec4::from((b[0], b[1], b[2], 1.));
                        let c = Vec4::from((c[0], c[1], c[2], 1.));

                        (vec![a, b, c], vec![0, 1, 2])
                    }
                    Shape::Square => {
                        let l = f32::sqrt(1. / 8.);
                        let a = Vec4::from((l, l, 0., 1.));
                        let b = Vec4::from((l, -l, 0., 1.));
                        let c = Vec4::from((-l, l, 0., 1.));
                        let d = Vec4::from((-l, -l, 0., 1.));

                        (vec![a, b, c, d], vec![0, 1, 2, 1, 2, 3])
                    }
                    Shape::Polygon(s) => {
                        let vx = (0..*s)
                            .map(|i| (i as f32 / *s as f32) * 2. * PI)
                            .map(|th| [f32::cos(th), f32::sin(th)])
                            .map(|[x, y]| Vec4::new(x / 2., y / 2., 0., 1.))
                            .collect();

                        let ix = (0..s - 2).flat_map(|i| [0, i + 1, i + 2]).collect();

                        (vx, ix)
                    }
                    Shape::Mesh(vx, tp) => {
                        let vx: Vec<Vec4> = vx.iter().map(|x| (*x, 1.0).into()).collect();

                        match tp {
                            Topology::TriangleList => {
                                //TODO: CHECK NUMBER OF VERTICES AND PANIC
                                let ix = (0..(vx.len() as u32)).collect();

                                (vx, ix)
                            }
                            Topology::TriangleStrip => {
                                //TODO: CHECK NUMBER OF VERTICES AND PANIC
                                let n = vx.len() as u32;

                                let ix = (0..n - 2).flat_map(|i| [i, i + 1, i + 2]).collect();

                                (vx, ix)
                            }
                        }
                    }
                };

                draws.push(DrawIndexedIndirect {
                    vertex_count: ix.len() as u32,
                    instance_count: 1,
                    base_index: indexes.len() as u32,
                    vertex_offset: vertexes.len() as i32,
                    base_instance: (instances.len() - 1) as u32,
                });

                vertexes.append(&mut vx);
                indexes.append(&mut ix);
            }

            node_stack.append(&mut curr.children.iter().rev().map(|x| x.clone()).collect());
            transform_index_stack.append(&mut vec![instances.len() - 1; curr.children.len()]);
        }

        let camera_view = Mat4::look_at_lh(
            self.camera.view.eye,
            self.camera.view.center,
            self.camera.view.up,
        );
        let camera_projection = match self.camera.projection {
            CameraProjection::Perspective {
                fov_y_radians,
                aspect_ratio,
                z_near,
                z_far,
            } => Mat4::perspective_lh(fov_y_radians, aspect_ratio, z_near, z_far),
            CameraProjection::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => Mat4::orthographic_lh(left, right, bottom, top, near, far),
        };

        let camera = camera_projection.mul_mat4(&camera_view);

        let clear = wgpu::Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };

        return RenderState {
            vertexes,
            indexes,
            instances,
            draws,
            clear,
            camera,
        };
    }
}

pub enum Shape {
    Triangle(Vec3, Vec3, Vec3),
    Square,
    Polygon(u32),
    Mesh(Vec<Vec3>, Topology),
}

pub enum Topology {
    TriangleList,
    TriangleStrip,
}

#[derive(Default)]
pub struct Camera {
    pub view: CameraView,
    pub projection: CameraProjection,
}

pub struct CameraView {
    pub(crate) eye: Vec3,
    pub(crate) center: Vec3,
    pub(crate) up: Vec3,
}

impl Default for CameraView {
    fn default() -> Self {
        Self {
            eye: Vec3::new(0., 0., 1.),
            center: Vec3::new(0., 0., 0.),
            up: Vec3::new(0., 1., 0.),
        }
    }
}

pub enum CameraProjection {
    Perspective {
        fov_y_radians: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

impl Default for CameraProjection {
    fn default() -> Self {
        Self::Perspective {
            fov_y_radians: 50.,
            aspect_ratio: 1.,
            z_near: 0.1,
            z_far: 2000.0,
        }
    }
}

#[derive(Default)]
pub struct Transform {
    pub scale: Option<Vec3>,
    pub rotation: Option<Quat>,
    pub translation: Option<Vec3>,
}

#[derive(Clone, Copy)]
pub struct Frame {
    pub count: u32,
    pub size: (u32, u32),
}

#[derive(Default)]
pub struct Node {
    pub transform: Option<Transform>,
    pub color: Option<Color>,
    pub children: Vec<Rc<RefCell<Node>>>,
    pub shapes: Vec<usize>,
}
