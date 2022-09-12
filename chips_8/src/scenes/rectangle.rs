use std::ffi::CString;

use crate::gfx::core::{ Object, Scene, render_object };
use crate::gfx::core::bindings as bindings;

pub fn create_rectangle_scene(gl: &bindings::Gl) -> Scene {
    let mut objects = Vec::with_capacity(1);

    let rectangle_vertices = [
        // positions       // colors     
         1.0,  1.0,  0.0,  1.0, 0.0, 0.0,
         1.0, -1.0,  0.0,  0.0, 1.0, 0.0,
        -1.0, -1.0,  0.0,  0.0, 1.0, 0.0,
        -1.0,  1.0,  0.0,  0.0, 0.0, 1.0
    ];

    let indices = [
        0, 1, 3,
        1, 2, 3
    ];

    let attributes = [
        (0, 3),
        (1, 3)
    ];

    let frag_src_raw = &CString::new(include_str!("../shaders/triangle.frag")).unwrap();
    let vert_src_raw = &CString::new(include_str!("../shaders/triangle.vert")).unwrap();

    let frag_src = Some(frag_src_raw.as_c_str());
    let vert_src = Some(vert_src_raw.as_c_str());
    let mut triangle = Object::new_with_shaders(gl, Box::new(rectangle_vertices), Some(Box::new(indices)), Box::new(attributes), frag_src, vert_src);
    
    match render_object(gl, &mut triangle) {
        Ok(_) => objects.push(triangle),
        Err(err) => panic!("{}", err)
    };

    Scene::new(objects.into_boxed_slice())
}