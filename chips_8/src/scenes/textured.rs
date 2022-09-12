use std::ffi::CString;

use crate::gfx::core::{Object, Scene, render_object};
use crate::gfx::core::bindings as bindings;

pub fn create_textured_scene(gl: &bindings::Gl) -> Scene {
    let mut objects = Vec::with_capacity(1); // I know there will only be one so no need to waste here

    let rectangle_verts = [
        // Positions       // Colors         // Texture coords
         0.5,  0.5,  0.0,  1.0,  0.0,  0.0,  1.0,  1.0, //  top right
         0.5, -0.5,  0.0,  0.0,  1.0,  0.0,  1.0,  0.0, //  bottom right
        -0.5, -0.5,  0.0,  0.0,  0.0,  1.0,  0.0,  0.0, //  bottom left
        -0.5,  0.5,  0.0,  1.0,  1.0,  0.0,  0.0,  1.0  //  top left
    ];

    let indices = [
        0, 1, 3,
        1, 2, 3
    ];

    let attributes = [
        (0, 3),
        (1, 3),
        (2, 2),
    ];
    let frag_src_raw = &CString::new(include_str!("../shaders/texture.frag")).unwrap();
    let vert_src_raw = &CString::new(include_str!("../shaders/texture.vert")).unwrap();

    let frag_src = Some(frag_src_raw.as_c_str());
    let vert_src = Some(vert_src_raw.as_c_str());

    let mut rectangle = Object::new_with_texture_shader(
        gl, 
        Box::new(rectangle_verts), 
        Some(Box::new(indices)), 
        Box::new(attributes), 
        frag_src, 
        vert_src,
        100,
        100,
        bindings::TEXTURE_2D,
        vec![
                        (bindings::TEXTURE_WRAP_S, bindings::CLAMP_TO_EDGE),
                        (bindings::TEXTURE_WRAP_T, bindings::CLAMP_TO_EDGE),
                        (bindings::TEXTURE_MIN_FILTER, bindings::LINEAR_MIPMAP_LINEAR),
                        (bindings::TEXTURE_MAG_FILTER, bindings::LINEAR),
                    ].into_boxed_slice()
    );

    if let Some(texture) = &mut rectangle.texture {
        (0..10)
            .for_each(|x| {
                (0..10).for_each(|y| {
                    texture.edit_texture_data(x, y, (100, 255, 0, 255))
                })
            });
    }

    match render_object(gl, &mut rectangle) {
        Ok(_) => objects.push(rectangle),
        Err(err) => panic!("{}", err)
    };

    Scene::new(objects.into_boxed_slice())
}