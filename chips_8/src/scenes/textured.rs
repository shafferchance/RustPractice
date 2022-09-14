use std::ffi::CString;

use crate::gfx::core::{Object, Scene, edit_texture, render_object};
use crate::gfx::core::bindings as bindings;

fn get_quad_verts() -> [f32; 32] {
    [
        // Positions       // Colors         // Texture coords
         0.5,  0.5,  0.0,  1.0,  0.0,  0.0,  1.0,  1.0, //  top right
         0.5, -0.5,  0.0,  0.0,  1.0,  0.0,  1.0,  0.0, //  bottom right
        -0.5, -0.5,  0.0,  0.0,  0.0,  1.0,  0.0,  0.0, //  bottom left
        -0.5,  0.5,  0.0,  1.0,  1.0,  0.0,  0.0,  1.0  //  top left
    ]
}

fn get_quad_indices() -> [i32; 6] {
    [
        0, 1, 3,
        1, 2, 3
    ]
}

fn get_quad_attribs() -> [(u32, i32); 3] {
    [
        (0, 3),
        (1, 3),
        (2, 2),
    ]
}

pub fn create_textured_scene(gl: &bindings::Gl) -> Scene {
    let mut objects = Vec::with_capacity(1); // I know there will only be one so no need to waste here

    let rectangle_verts = get_quad_verts();

    let indices = get_quad_indices();

    let attributes = get_quad_attribs();
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

    // (100, 255, 0, 255)
    if let Some(texture) = &mut rectangle.texture {
        // Create pixel array
        let pixel_array = [(100 as u8, 255 as u8, 0 as u8, 255 as u8); 100].iter().flat_map(|pixel_t| {
            return [pixel_t.0, pixel_t.1, pixel_t.2, pixel_t.3]
        }).collect::<Vec<u8>>();
        edit_texture(texture, (0, 10), (0, 10), &pixel_array.as_slice())
    }

    match render_object(gl, &mut rectangle) {
        Ok(_) => objects.push(rectangle),
        Err(err) => panic!("{}", err)
    };

    Scene::new(objects.into_boxed_slice())
}

// Little too specific for me but oh well
// Yes this violates DRY to a degree but I don't feel like fighting the borrow checker right now
pub fn create_scene_with_chips_8_text(gl: &bindings::Gl, pixels: &[u8; 64 * 32]) -> Scene {
    let mut objects = Vec::with_capacity(1); // I know there will only be one so no need to waste here

    let rectangle_verts = get_quad_verts();

    let indices = get_quad_indices();

    let attributes = get_quad_attribs();
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
        64,
        64, // The real height is 32, but OpenGL needs them to be the same
        bindings::TEXTURE_2D,
        vec![
                        (bindings::TEXTURE_WRAP_S, bindings::CLAMP_TO_EDGE),
                        (bindings::TEXTURE_WRAP_T, bindings::CLAMP_TO_EDGE),
                        (bindings::TEXTURE_MIN_FILTER, bindings::LINEAR_MIPMAP_LINEAR),
                        (bindings::TEXTURE_MAG_FILTER, bindings::LINEAR),
                    ].into_boxed_slice()
    );

    if let Some(texture) = &mut rectangle.texture {
        (0..64)
            .for_each(|x| {
                (0..32).for_each(|y| {
                    let value_at_pixel = pixels[(63 - x) * (31 -y) as usize];
                    
                    texture.edit_texture_data(x, y, match value_at_pixel {
                        1 => (255, 255, 255, 255),
                        0 => (0, 0, 0, 255),
                        _ => (255, 0, 0, 255)  // Marker for pixels at incorrect value
                    })
                })
            });
    }

    match render_object(gl, &mut rectangle) {
        Ok(_) => objects.push(rectangle),
        Err(err) => panic!("{}", err)
    };

    Scene::new(objects.into_boxed_slice())
}