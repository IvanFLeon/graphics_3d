use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use glam::{Quat, Vec3};

use crate::state::{Camera, CameraProjection, CameraView, Node, Shape, State, Topology, Transform};

pub struct App {
    prev_node: Rc<RefCell<Node>>,
    curr_node: Rc<RefCell<Node>>,
    pub(crate) state: State,
    pub frame: u32,
    pub size: Size,
}

pub struct Size {
    pub width: u32,
    pub height: u32,
}

pub struct Mesh {
    vertex: Vec<Vec3>,
}

impl Mesh {
    pub fn vertex(&mut self, v: [f32; 3]) {
        self.vertex.push(v.into());
    }
}

impl App {
    pub fn new(width: u32, height: u32) -> Self {
        //Set initial camera for screen size
        let fov = PI / 3.;
        let camera_z = ((height as f32) / 2.) / f32::tan(fov);
        let camera = Camera {
            view: CameraView {
                eye: Vec3::new(0., 0., -camera_z),
                center: Vec3::new(0., 0., 0.),
                up: Vec3::new(0., 1., 0.),
            },
            projection: CameraProjection::Perspective {
                fov_y_radians: fov,
                aspect_ratio: width as f32 / height as f32,
                z_near: camera_z / 10.,
                z_far: camera_z * 10.,
            },
        };

        let state = State {
            camera,
            ..Default::default()
        };

        Self {
            size: Size { width, height },
            prev_node: state.root.clone(),
            curr_node: state.root.clone(),
            state,
            frame: 0,
        }
    }

    pub fn orthographic(
        &mut self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) {
        self.state.camera.projection = CameraProjection::Orthographic {
            left,
            right,
            bottom,
            top,
            near,
            far,
        }
    }

    pub fn triangle(&mut self, a: [f32; 3], b: [f32; 3], c: [f32; 3]) {
        let mut curr = self.curr_node.borrow_mut();

        let triangle = Shape::Triangle(a.into(), b.into(), c.into());

        self.state.shapes.push(triangle);

        curr.shapes.push(self.state.shapes.len() - 1);
    }

    pub fn square(&mut self) {
        let mut curr = self.curr_node.borrow_mut();

        let square = Shape::Square;

        self.state.shapes.push(square);

        curr.shapes.push(self.state.shapes.len() - 1);
    }

    pub fn polygon(&mut self, n: u32) {
        let mut curr = self.curr_node.borrow_mut();

        self.state.shapes.push(Shape::Polygon(n));

        curr.shapes.push(self.state.shapes.len() - 1);
    }

    pub fn circle(&mut self) {
        let mut curr = self.curr_node.borrow_mut();

        self.state.shapes.push(Shape::Polygon(80));

        curr.shapes.push(self.state.shapes.len() - 1);
    }

    pub fn mesh(&mut self, topology: Topology, f: impl Fn(&mut Mesh) -> ()) {
        let mut mesh = Mesh { vertex: vec![] };

        f(&mut mesh);

        let mut curr = self.curr_node.borrow_mut();

        self.state.shapes.push(Shape::Mesh(mesh.vertex, topology));

        curr.shapes.push(self.state.shapes.len() - 1);
    }

    // pub fn line(&mut self, length: u32, width: u32) {
    //     self.push(Some([width, length, 1.]), , )

    // }

    pub fn push(
        &mut self,
        scale: Option<[f32; 3]>,
        rotation: Option<[f32; 4]>,
        translation: Option<[f32; 3]>,
        color: Option<[f64; 4]>,
    ) {
        let mut transform = Transform::default();

        if let Some(scale) = scale {
            transform.scale = Some(Vec3::from_array(scale));
        };
        if let Some(rotation) = rotation {
            transform.rotation = Some(Quat::from_array(rotation));
        };
        if let Some(translation) = translation {
            transform.translation = Some(Vec3::from_array(translation));
        };

        self.prev_node = self.curr_node.clone();

        let mut prev = self.prev_node.borrow_mut();

        let curr = Rc::new(RefCell::new(Node {
            transform: Some(transform),
            color: color.map(|c| c.into()),
            children: Vec::new(),
            shapes: Vec::new(),
        }));

        prev.children.push(curr.clone());

        self.curr_node = curr.clone();
    }

    pub fn pop(&mut self) {
        self.curr_node = self.prev_node.clone();
    }
}
