use std::ffi::CString;

use crate::gfx::init::{Object, Scene};
use crate::gfx::init::bindings as bindings;

pub fn create_triangle_scene(gl: &bindings::Gl) -> Scene {
    let mut objects = Vec::with_capacity(1); // I know there will only be one so no need to waste here

    let triangle_verts = [
        // Positions       // Colors
        -0.5, -0.5,  0.0,  1.0,  0.0,  0.0, 
         0.5, -0.5,  0.0,  0.0,  1.0,  0.0,
         0.0,  0.5,  0.0,  0.0,  0.0,  1.0
    ];

    let attributes = [
        (0, 3),
        (0, 3)
    ];

    let frag_src_raw = &CString::new(include_str!("../shaders/triangle.frag")).unwrap();
    let vert_src_raw = &CString::new(include_str!("../shaders/triangle.vert")).unwrap();

    let frag_src = Some(frag_src_raw.as_c_str());
    let vert_src = Some(vert_src_raw.as_c_str());

    let triangle = Object::new_with_shaders(gl, Box::new(triangle_verts), None, Box::new(attributes), frag_src, vert_src);
    objects.push(triangle);

    Scene::new(objects.into_boxed_slice())
}