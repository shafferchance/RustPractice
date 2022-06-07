use glutin::{self, PossiblyCurrent};

use std::{ffi::CStr, os::raw::c_char};

pub mod gl {
    pub use self::Gles2 as Gl;
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub struct Gl {
    pub gl: gl::Gl,
}

pub fn get_c_string_from_data(data: &[u8]) -> Vec<u8> {
    [data.as_ref(), b"\0"].concat()
}

pub fn load_vertex_shader(gl: &gl::Gl, shader_src: &[u8]) -> gl::types::GLuint {
    unsafe {
        let shader_src = get_c_string_from_data(shader_src);
        let vs = gl.CreateShader(gl::VERTEX_SHADER);
        gl.ShaderSource(vs, 1, [shader_src.as_ptr() as *const _].as_ptr(), std::ptr::null());
        gl.CompileShader(vs);
        
        vs
    }
}

pub fn load_fragment_shader(gl: &gl::Gl, shader_src: &[u8]) -> gl::types::GLuint {
    unsafe {
       let shader_src = get_c_string_from_data(shader_src);
       let fs = gl.CreateShader(gl::FRAGMENT_SHADER);
       gl.ShaderSource(fs, 1, [shader_src.as_ptr() as *const _].as_ptr(), std::ptr::null());
       gl.CompileShader(fs);
    
       fs
   }
}

pub fn create_program(gl: &gl::Gl) -> gl::types::GLuint {
    unsafe { gl.CreateProgram() }
}

pub fn load_vertex_shader_data<'a>(gl: &'a gl::Gl, vertex_buffer: &[f32]) {
    unsafe {
        let mut vb = std::mem::zeroed();
        gl.GenBuffers(1, &mut vb);
        gl.BindBuffer(gl::ARRAY_BUFFER, vb);
        gl.BufferData(
            gl::ARRAY_BUFFER,
            (vertex_buffer.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertex_buffer.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
    }
}

// TODO: Need to figure out what this is...
pub fn bind_vertex_array(gl: &gl::Gl) {
    if gl.BindVertexArray.is_loaded() {
        unsafe {
            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);
        };
    }
}

pub fn bind_elements_buffer(gl: &gl::Gl, data: &[u8]) {
    unsafe {
        let mut ebo = std::mem::zeroed();
        gl.GenBuffers(1, &mut ebo);
        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl.BufferData(gl::ELEMENT_ARRAY_BUFFER, (data.len() * std::mem::size_of::<u8>()) as gl::types::GLsizeiptr, data.as_ptr() as *const _, gl::STATIC_DRAW);
    }
}

pub fn add_attribute(
    gl: &gl::Gl, 
    program: &gl::types::GLuint, 
    attribute_name: &[u8],
    size: i32,
    attribute_type: gl::types::GLenum,
    stride: i32,                        // This should be constant, at-least it seems?
    pointer_offset: usize               // This is the offset within the Array Buffer!
) {
    unsafe {
        let pointer = if pointer_offset == 0 { std::ptr::null() } else { pointer_offset as *const () as *const _ };
        let attribute_location = gl.GetAttribLocation(*program, attribute_name.as_ptr() as *const _);
        gl.VertexAttribPointer(
            attribute_location as gl::types::GLuint, 
            size, 
            attribute_type, 
            0, 
            stride as gl::types::GLsizei, 
            pointer
        );
        gl.EnableVertexAttribArray(attribute_location as gl::types::GLuint);
    };
}

pub fn attach_shader(gl: &gl::Gl, program: &gl::types::GLuint, shader_data: gl::types::GLuint) {
    unsafe {
        gl.AttachShader(*program, shader_data)
    }
}

pub fn finalize_shaders(gl: &gl::Gl, program: &gl::types::GLuint) {
    unsafe {
        gl.LinkProgram(*program);
        gl.UseProgram(*program);
    }
}

pub fn load(gl_context: &glutin::Context<PossiblyCurrent>) -> Gl {
    let gl = gl::Gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);

    let version = unsafe {
        let data = CStr::from_ptr(gl.GetString(gl::VERSION) as *const _).to_bytes().to_vec();
        String::from_utf8(data).unwrap()
    };

    println!("OpenGL version {}", version);

    let vs = load_vertex_shader(&gl, include_bytes!("../shaders/triangle.vert"));

    let fs = load_fragment_shader(&gl, include_bytes!("../shaders/triangle.frag"));

    let program = create_program(&gl);
    attach_shader(&gl, &program, vs);
    attach_shader(&gl, &program, fs);
    finalize_shaders(&gl, &program);

    load_vertex_shader_data(&gl, &[
         0.5,  0.5,  0.0,
         0.5, -0.5,  0.0,
        -0.5, -0.5,  0.0,
        -0.5,  0.5,  0.0,
    ]);



    bind_vertex_array(&gl);
    let indices = vec![
        0, 1, 3,
        1, 2, 3,
    ];

    bind_elements_buffer(&gl, &indices);


    let stride = 5 * std::mem::size_of::<f32>() as i32;
    add_attribute(&gl, &program, b"position\0", 2, gl::FLOAT, stride, 0);
    add_attribute(&gl, &program, b"color\0", 3, gl::FLOAT, stride, 2 * std::mem::size_of::<f32>());

    Gl { gl }
}

impl Gl {
    pub fn draw_frame(&self, color: [f32; 4]) {
        unsafe {
            self.gl.ClearColor(color[0], color[1], color[2], color[3]);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.gl.DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
        }
    }
}
