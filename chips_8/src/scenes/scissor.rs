use std::ffi::CString;

use crate::gfx::init::{Object, Scene, render_object};
use crate::gfx::init::bindings as bindings;

pub fn create_scissor_scene(gl: &bindings::Gl) -> Scene {
    let mut objects = Vec::with_capacity(1);

    let shape_vertices = [
        1.0,  1.0,  0.0,
        1.0, -1.0,  0.0,
       -1.0, -1.0,  0.0,
       -1.0,  1.0,  0.0,
    ];

    let indices = [
        0, 1, 3,
        1, 2, 3
    ];

    let frag_src_raw = &CString::new(include_str!("../shaders/scissor.frag")).unwrap();
    let vert_src_raw = &CString::new(include_str!("../shaders/scissor.vert")).unwrap();

    let frag_src = Some(frag_src_raw.as_c_str());
    let vert_src = Some(vert_src_raw.as_c_str());

    let mut rectangle = Object::new_with_shaders(
        gl,
        Box::new(shape_vertices),
        Some(Box::new(indices)),
        Box::new([]),
        frag_src,
        vert_src,
    );

    match render_object(gl, &mut rectangle) {
        Ok(_) => objects.push(rectangle),
        Err(err) => panic!("{}", err)
    };

    Scene::new(objects.into_boxed_slice())
}
