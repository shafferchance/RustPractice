use glutin::{self, PossiblyCurrent};

use std::rc::Rc;
use std::ops::Deref;
use std::ffi::{ CStr, CString };

mod bindings {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub use self::bindings::Gl as InnerGl;

#[derive(Clone)]
pub struct Gl {
    inner: Rc<bindings::Gl>,
}

impl Gl {
    pub fn load_with<F>(load_fn: F) -> Gl
        where F: FnMut(&'static str) -> *const bindings::types::GLvoid
    {
        Gl { inner: Rc::new(bindings::Gl::load_with(load_fn)) }    
    }
}

pub fn get_c_string_from_data(data: &[u8]) -> Vec<u8> {
    [data.as_ref(), b"\0"].concat()
}

impl Deref for Gl {
    type Target = bindings::Gl;

    fn deref(&self) -> &bindings::Gl {
        &self.inner
    }
}

pub fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}

pub fn shader_from_src (
    gl: &bindings::Gl,
    source: &CStr,
    kind: bindings::types::GLenum
) -> Result<bindings::types::GLuint, String> {
    let id = unsafe { gl.CreateShader(kind) };

    unsafe {
        gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl.CompileShader(id);
    }

    let mut success: bindings::types::GLint = 1;
    unsafe {
        gl.GetShaderiv(id, bindings::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: bindings::types::GLint = 0;
        unsafe {
            gl.GetShaderiv(id, bindings::INFO_LOG_LENGTH, &mut len);
        }
        let error = create_whitespace_cstring_with_len(len as usize);
        unsafe {
            gl.GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut bindings::types::GLchar
            );
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

pub struct Shader {
    gl: bindings::Gl,
    id: bindings::types::GLuint,
}

impl Shader {
    pub fn from_source(
        gl: &bindings::Gl,
        source: &CStr,
        kind: bindings::types::GLenum
    ) -> Result<Shader, String> {
        let id = shader_from_src(gl, source, kind)?;
        Ok(Shader { gl: gl.clone(), id })
    }

    pub fn from_vert_source(gl: &bindings::Gl, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, bindings::VERTEX_SHADER)
    }

    pub fn from_frag_source(gl: &bindings::Gl, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, bindings::FRAGMENT_SHADER)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

pub fn create_program(gl: &bindings::Gl) -> bindings::types::GLuint {
    unsafe { gl.CreateProgram() }
}

pub struct Program {
    gl: bindings::Gl,
    id: bindings::types::GLuint,
}

impl Program {
    pub fn from_shaders(gl: &bindings::Gl, shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl.CreateProgram() };

        for shader in shaders {
            unsafe { gl.AttachShader(program_id, shader.id); }
        }

        unsafe { gl.LinkProgram(program_id); }

        let mut success: bindings::types::GLint = 1;
        unsafe {
            gl.GetProgramiv(program_id, bindings::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: bindings::types::GLint = 0;
            unsafe {
                gl.GetProgramiv(program_id, bindings::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl.GetProgramInfoLog(program_id, len, std::ptr::null_mut(), error.as_ptr() as *mut bindings::types::GLchar);
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl.DetachShader(program_id, shader.id); }
        }

        Ok(Program { gl: gl.clone(), id: program_id })
    }

    pub fn set_used(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

pub fn load_elements_shader_data(gl: &bindings::Gl, vertex_buffer: &[f32], indices: &[i32]) -> (bindings::types::GLuint, bindings::types::GLuint, bindings::types::GLuint) {
    let mut vbo: bindings::types::GLuint = 0;
    let mut vao: bindings::types::GLuint = 0;
    let mut ebo: bindings::types::GLuint = 0;
    unsafe {
        // Generated buffers
        gl.GenVertexArrays(1, &mut vao);
        gl.GenBuffers(1, &mut vbo);
        gl.GenBuffers(1, &mut ebo);

        // Bind and fill buffers
        gl.BindVertexArray(vao);

        gl.BindBuffer(bindings::ARRAY_BUFFER, vbo);
        gl.BufferData(
            bindings::ARRAY_BUFFER,
            (vertex_buffer.len() * std::mem::size_of::<f32>()) as bindings::types::GLsizeiptr,
            vertex_buffer.as_ptr() as *const bindings::types::GLvoid,
            bindings::STATIC_DRAW,
        );
        gl.BindBuffer(bindings::ELEMENT_ARRAY_BUFFER, ebo);
        gl.BufferData(
            bindings::ELEMENT_ARRAY_BUFFER, 
            (indices.len() * std::mem::size_of::<i32>()) as bindings::types::GLsizeiptr, 
            indices.as_ptr() as *const bindings::types::GLvoid, 
            bindings::STATIC_DRAW
        );
        // Since setting these externally we cannot unbind either
        // gl.BindBuffer(bindings::ARRAY_BUFFER, 0);
        // gl.BindVertexArray(0);
    }

    return (vbo, vao, ebo);
}

pub fn load_vertex_shader_data(gl: &bindings::Gl, vertex_buffer: &[f32]) -> (bindings::types::GLuint, bindings::types::GLuint) {
    let mut vbo: bindings::types::GLuint = 0;
    let mut vao: bindings::types::GLuint = 0;
    unsafe {
        gl.GenVertexArrays(1, &mut vao);
        gl.BindVertexArray(vao);
        // gl.BindBuffer(bindings::ARRAY_BUFFER, 0);
    };

    unsafe {
        gl.GenBuffers(1, &mut vbo);
        gl.BindBuffer(bindings::ARRAY_BUFFER, vbo);
        gl.BufferData(
            bindings::ARRAY_BUFFER,
            (vertex_buffer.len() * std::mem::size_of::<f32>()) as bindings::types::GLsizeiptr,
            vertex_buffer.as_ptr() as *const bindings::types::GLvoid,
            bindings::STATIC_DRAW,
        );
    }

    return (vbo, vao);
}

pub fn bind_elements_buffer(gl: &bindings::Gl, data: &[u8]) {
    unsafe {
        let mut ebo = std::mem::zeroed();
        gl.GenBuffers(1, &mut ebo);
        gl.BindBuffer(bindings::ELEMENT_ARRAY_BUFFER, ebo);
        gl.BufferData(bindings::ELEMENT_ARRAY_BUFFER, (data.len() * std::mem::size_of::<u8>()) as bindings::types::GLsizeiptr, data.as_ptr() as *const _, bindings::STATIC_DRAW);
    }
}

pub fn add_attribute(
    gl: &bindings::Gl, 
    // program: &bindings::types::GLuint, 
    attribute_location: bindings::types::GLuint,
    size: i32,
    attribute_type: bindings::types::GLenum,
    stride: bindings::types::GLsizei,                        // This should be constant, at-least it seems?
    pointer_offset: *const bindings::types::GLvoid,               // This is the offset within the Array Buffer!
) {
    unsafe {
        // let attribute_location = gl.GetAttribLocation(*program, attribute_name.as_ptr() as *const _);
        gl.EnableVertexAttribArray(attribute_location);
        gl.VertexAttribPointer(
            attribute_location, 
            size, 
            attribute_type, 
            0, 
            stride, 
            pointer_offset
        );
    };
}

pub fn attach_shader(gl: &bindings::Gl, program: &bindings::types::GLuint, shader_data: bindings::types::GLuint) {
    unsafe {
        gl.AttachShader(*program, shader_data)
    }
}

pub fn finalize_shaders(gl: &bindings::Gl, program: &bindings::types::GLuint) {
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
    let gl = Gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const std::os::raw::c_void);

    let version = unsafe {
        let data = CStr::from_ptr(gl.GetString(bindings::VERSION) as *const _).to_bytes().to_vec();
        String::from_utf8(data).unwrap()
    };

    println!("OpenGL version {}", version);
    let vs = Shader::from_vert_source(&gl, &CString::new(include_str!("../shaders/triangle.vert")).unwrap()).unwrap();
    let fs = Shader::from_frag_source(&gl, &CString::new(include_str!("../shaders/triangle.frag")).unwrap()).unwrap();
    let program = Program::from_shaders(&gl, &[vs, fs]).unwrap();
    program.set_used();

//     let triangle_verts = [
//         // Positions       // Colors
//         -0.5, -0.5,  0.0,  1.0,  0.0,  0.0, 
//          0.5, -0.5,  0.0,  0.0,  1.0,  0.0,
//          0.0,  0.5,  0.0,  0.0,  0.0,  1.0
//    ];

    let rectangle_vertices = [
         0.5,  0.5,  0.0,  1.0, 0.0, 0.0,
         0.5, -0.5,  0.0,  0.0, 1.0, 0.0,
        -0.5, -0.5,  0.0,  0.0, 1.0, 0.0,
        -0.5,  0.5,  0.0,  0.0, 0.0, 1.0
    ];

    let indices = [
        0, 1, 3,
        1, 2, 3
    ];
    // load_vertex_shader_data(&gl, &triangle_verts);
    let (_vbo, _vao, _ebo) = load_elements_shader_data(&gl, &rectangle_vertices, &indices);
    add_attribute(&gl, 0, 3, bindings::FLOAT, 6 * std::mem::size_of::<f32>() as bindings::types::GLint, 0 as *const bindings::types::GLvoid);
    add_attribute(&gl, 1, 3, bindings::FLOAT, 6 * std::mem::size_of::<f32>() as bindings::types::GLint, (3 * std::mem::size_of::<f32>()) as *const bindings::types::GLvoid);
    unsafe {
        gl.BindBuffer(bindings::ARRAY_BUFFER, 0);
        // gl.BindVertexArray(vao);
    }


    return gl;
}

impl Gl {
    pub fn draw_frame(&self, color: [f32; 4]) {
        unsafe {
            self.inner.ClearColor(color[0], color[1], color[2], color[3]);
            self.inner.Clear(bindings::COLOR_BUFFER_BIT);
            // self.inner.DrawArrays(
            //     bindings::TRIANGLES,
            //     0,
            //     3
            // );
            self.inner.DrawElements(
                bindings::TRIANGLES, 
                6 as bindings::types::GLint,
                bindings::UNSIGNED_INT,
                std::ptr::null() as *const bindings::types::GLvoid);
        }
    }
}
