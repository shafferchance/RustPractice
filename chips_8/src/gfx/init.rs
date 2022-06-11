use glutin::{self, PossiblyCurrent};

use std::ffi::{ CStr, CString };

pub mod gl {
    pub use self::Gl as OtherGl;
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub struct Gl {
    pub gl: gl::Gl,
}

pub fn get_c_string_from_data(data: &[u8]) -> Vec<u8> {
    [data.as_ref(), b"\0"].concat()
}

pub fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}

pub fn shader_from_src (
    gl: &gl::Gl,
    source: &[u8],
    kind: gl::types::GLenum
) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl.CreateShader(kind) };

    unsafe {
        let shader_src = get_c_string_from_data(source);
        gl.ShaderSource(id, 1, [shader_src.as_ptr() as *const _].as_ptr(), std::ptr::null());
        gl.CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }
        let error = create_whitespace_cstring_with_len(len as usize);
        unsafe {
            gl.GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar
            );
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

pub struct Shader<'a> {
    gl: &'a gl::Gl,
    id: gl::types::GLuint,
}

impl<'a> Shader<'a> {
    pub fn from_source(
        gl: &'a gl::Gl,
        source: &[u8],
        kind: gl::types::GLenum
    ) -> Result<Shader<'a>, String> {
        let id = shader_from_src(gl, source, kind)?;
        Ok(Shader { gl, id })
    }

    pub fn from_vert_source(gl: &'a gl::Gl, source: &[u8]) -> Result<Shader<'a>, String> {
        Shader::from_source(gl, source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(gl: &'a gl::Gl, source: &[u8]) -> Result<Shader<'a>, String> {
        Shader::from_source(gl, source, gl::FRAGMENT_SHADER)
    }
}

impl<'a> Drop for Shader<'a> {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
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

pub fn get_cstr_from_bytes(file_bytes: &[u8]) -> &CStr {
    match CStr::from_bytes_with_nul(file_bytes) {
        Ok(data) => data,
        Err(err) => panic!("{}", err)
    }
}

pub fn load(gl_context: &glutin::Context<PossiblyCurrent>) -> Gl {
    let gl = gl::Gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);

    let version = unsafe {
        let data = CStr::from_ptr(gl.GetString(gl::VERSION) as *const _).to_bytes().to_vec();
        String::from_utf8(data).unwrap()
    };

    println!("OpenGL version {}", version);
    let vs = match Shader::from_vert_source(&gl, include_bytes!("../shaders/triangle.vert")) {
        Ok(shader) => shader,
        Err(err) => panic!("{}", err)
    };

    let fs = match Shader::from_frag_source(&gl, include_bytes!("../shaders/triangle.frag")) {
        Ok(shader) => shader,
        Err(err) => panic!("{}",err)
    };

    let program = create_program(&gl);
    attach_shader(&gl, &program, vs.id);
    attach_shader(&gl, &program, fs.id);
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

    Gl { gl: gl.to_owned() }
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
