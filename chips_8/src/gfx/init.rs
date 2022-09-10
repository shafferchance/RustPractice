use glutin::{self, PossiblyCurrent};

use std::rc::Rc;
use std::ops::Deref;
use std::ffi::{ CStr, CString };

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

use self::bindings::ARRAY_BUFFER;
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
    pub fn from_shaders(gl: &bindings::Gl, shaders: Box<[Shader]>) -> Result<Program, String> {
        let program_id = unsafe { gl.CreateProgram() };

        for shader in shaders.into_iter() {
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

        for shader in shaders.into_iter() {
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

pub fn load_elements_shader_data(gl: &bindings::Gl, vertex_buffer: &Box<[f32]>, indices: &Box<[i32]>) -> (bindings::types::GLuint, bindings::types::GLuint, bindings::types::GLuint) {
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
    }

    return (vbo, vao, ebo);
}

pub fn load_vertex_shader_data(gl: &bindings::Gl, vertex_buffer: &Box<[f32]>) -> (bindings::types::GLuint, bindings::types::GLuint) {
    let mut vbo: bindings::types::GLuint = 0;
    let mut vao: bindings::types::GLuint = 0;
    unsafe {
        gl.GenVertexArrays(1, &mut vao);
        gl.GenBuffers(1, &mut vbo);

        gl.BindVertexArray(vao);
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

type PixelValue = (u8, u8, u8, u8);
type TextureAttribute = (bindings::types::GLenum, bindings::types::GLenum);

// Inefficient yes, but we're going to try!
pub struct Texture {
    pub pixels: Box<[u8]>,
    texture_type: bindings::types::GLenum,
    id: bindings::types::GLuint,
    width: usize,
    height: usize,
    gl: bindings::Gl,
    attributes: Box<[TextureAttribute]>
}

impl Texture {
    pub fn new (gl: &bindings::Gl, width_size: usize, height_size: usize, texture_type: bindings::types::GLenum, attributes: Box<[TextureAttribute]>) -> Texture {
        let mut inner_index = 1;
        let pixels = 
            (0..((width_size * height_size) * 4))
                .map(|_| {
                    if inner_index == 4 {
                        inner_index = 1;
                        return 255;
                    }
                    inner_index += 1;

                    0 as u8
                })
                .collect::<Vec<u8>>()
                .into_boxed_slice();
        let mut id: bindings::types::GLuint = 0;
        unsafe {
            gl.GenTextures(1, &mut id);
        };

        Texture { pixels, id, texture_type, width: width_size, height: height_size, attributes, gl: gl.clone()}
    }

    fn set_texture_parameter(&self, parameter_name: bindings::types::GLenum, param: bindings::types::GLint) {
        unsafe {
            // Need to ensure we're operating on the right texture
            self.gl.TexParameteri(self.texture_type, parameter_name, param);
        }
    }

    pub fn edit_texture_data(&mut self, x: usize, y: usize, pixel_data: PixelValue) {
        // Assuming this is row driven
        let index = ((self.width * y) + x) * 4;
        // println!("{}", index);
        self.pixels[index]     = pixel_data.0;
        self.pixels[index + 1] = pixel_data.1;
        self.pixels[index + 2] = pixel_data.2;
        self.pixels[index + 3] = pixel_data.3;
    }

    pub fn load_texture_data(&self, ) {
        unsafe {
            self.gl.BindTexture(self.texture_type, 0);
            self.gl.BindTexture(self.texture_type, self.id);
            // This will need an external function to figure out which to use eventually...
            self.gl.TexImage2D(
                self.texture_type, 
                0, 
                bindings::RGBA as bindings::types::GLint, 
                self.width as i32, 
                self.height as i32, 
                0, 
                bindings::RGBA, 
                bindings::UNSIGNED_BYTE, 
                self.pixels.as_ptr() as *const bindings::types::GLvoid
            );
            self.gl.GenerateMipmap(self.texture_type);
            self.attributes.into_iter().for_each(|attribute| {
                self.set_texture_parameter(attribute.0, attribute.1 as bindings::types::GLint);
            });
            self.gl.BindTexture(self.texture_type, 0);
        }
    }
}

// impl Drop for Texture {
//     fn drop(&mut self) {
//         unsafe { self.gl.DeleteTextures(1, self.id as *const bindings::types::GLuint);  }
//     }
// }

pub fn load_gl(gl_context: &glutin::Context<PossiblyCurrent>) -> Gl {
    let gl = Gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const std::os::raw::c_void);

    let version = unsafe {
        let data = CStr::from_ptr(gl.GetString(bindings::VERSION) as *const _).to_bytes().to_vec();
        String::from_utf8(data).unwrap()
    };

    println!("OpenGL version {}", version);

    gl
}

type Attributes = (bindings::types::GLuint, i32);
                      // Vertex Buffer Object  // Vertex Array Object   // Element Buffer Object
type ObjectBuffers = (bindings::types::GLuint, bindings::types::GLuint, Option<bindings::types::GLuint>);

pub struct Object {
    vertices: Box<[f32]>,
    indices: Option<Box<[i32]>>,
    // 0.attribute location 1. size (This assumes all values are floats currently...)
    attributes: Box<[Attributes]>,
    pub texture: Option<Texture>,
    program: Option<Program>,
    // This will likely become a Hashmap later
    buffers: Option<ObjectBuffers>,
    stride_length: i32
}

// TODO: Add support for geometry shaders
fn init_object_shaders(gl: &bindings::Gl, frag_src: Option<&CStr>, vert_src: Option<&CStr>) -> Box<[Shader]> {
    // vert, frag, and geo shaders
    let mut shaders = Vec::<Shader>::with_capacity(3);

    if let Some(fragment_shader) = frag_src {
        shaders.push(Shader::from_frag_source(gl, fragment_shader).unwrap());
    };

    if let Some(vertex_shader) = vert_src {
        shaders.push(Shader::from_vert_source(gl, vertex_shader).unwrap());
    };

    shaders.into_boxed_slice()
}

impl Object {
    pub fn new(vertices: Box<[f32]>, indices: Option<Box<[i32]>>, attributes: Box<[Attributes]>) -> Object {
        Object { vertices, attributes, indices, texture: None, program: None, buffers: None, stride_length: 0 }
    }

    pub fn new_with_shaders(gl: &bindings::Gl, vertices: Box<[f32]>, indices: Option<Box<[i32]>>, attributes: Box<[Attributes]>, frag_src: Option<&CStr>, vert_src: Option<&CStr>) -> Object {
        let shaders = init_object_shaders(gl, frag_src, vert_src);
        let program = match Program::from_shaders(gl, shaders) {
            Ok(shader) => Some(shader),
            Err(err) => panic!("{}", err)
        };

        Object { vertices, attributes, indices,  program, texture: None, buffers: None, stride_length: 0 }
    }

    pub fn new_with_texture_shader(gl: &bindings::Gl, vertices: Box<[f32]>, indices: Option<Box<[i32]>>, attributes: Box<[Attributes]>, frag_src: Option<&CStr>, vert_src: Option<&CStr>, width: usize, height: usize, texture_type: bindings::types::GLenum, texture_attributes: Box<[TextureAttribute]>) -> Object {
        let shaders = init_object_shaders(gl, frag_src, vert_src);
        let program = match Program::from_shaders(gl, shaders) {
            Ok(shader) => Some(shader),
            Err(err) => panic!("{}", err)
        };

        let texture = 
            Some(
                Texture::new(
                    gl, 
                    width, 
                    height, 
                    texture_type,
                    texture_attributes
                ));

        Object { vertices, attributes, texture, program, indices, buffers: None, stride_length: 0 }
    }
}

pub struct Scene {
    objects: Box<[Object]>,
}

impl Scene {
    pub fn new( objects: Box<[Object]>) -> Scene {
        Scene { objects }
    }
}

fn unbind_buffers(gl: &bindings::Gl) {
    unsafe {
        gl.BindBuffer(ARRAY_BUFFER, 0);
        gl.BindVertexArray(0);
    }
}

fn get_stride_from_attributes_tuple(attributes: &Box<[Attributes]>) -> (i32, bindings::types::GLint) {
    let stride_length = attributes.into_iter().map(| attrib | attrib.1).sum::<i32>();
    (stride_length, stride_length * std::mem::size_of::<f32>() as bindings::types::GLint)
}

pub fn render_object(gl: &bindings::Gl, object: &mut Object) -> Result<(), String> {
    let buffers = 
        match &object.indices {
            Some(indices_array) => {
                let (vbo, vao, ebo) = load_elements_shader_data(gl, &object.vertices, indices_array);
                (vbo, vao, Some(ebo))
            },
            None => {
                let (vbo, vao) = load_vertex_shader_data(gl, &object.vertices);
                (vbo, vao, None)
            }
    };

    println!("{:?}", buffers);

    object.buffers = Some(buffers);

    let (stride_length, stride) = get_stride_from_attributes_tuple(&object.attributes);
    object.stride_length = stride_length;
    object.attributes.into_iter().fold(0, |acc, &attribute| {
        add_attribute(gl, attribute.0, attribute.1, bindings::FLOAT, stride, (acc * std::mem::size_of::<f32>()) as *const bindings::types::GLvoid);
        acc + attribute.1 as usize
    });

    if let Some(texture) = &object.texture {
        texture.load_texture_data();
    }

    unbind_buffers(gl);

    // Might initialize an object that doesn't use a shader
    if let Some(program) = &object.program {
        program.set_used();
    }

    Ok(())
}

pub fn render_scene(gl: &bindings::Gl, scene: &Scene) {
    scene.objects.into_iter().for_each(| object | {
        if let Some(texture) = &object.texture {
            unsafe { gl.BindTexture(texture.texture_type, texture.id) };
        }

        if let Some(shader) = &object.program {
            shader.set_used();
        }

        if let Some(buffers) = &object.buffers {
            unsafe { gl.BindVertexArray(buffers.1) }

            if let Some(_) = buffers.2 {
                unsafe { gl.DrawElements(bindings::TRIANGLES, *&object.indices.as_ref().unwrap().len() as bindings::types::GLint, bindings::UNSIGNED_INT, 0 as *const bindings::types::GLvoid); }
            } else {
                unsafe { gl.DrawArrays(bindings::TRIANGLES, 0, (*&object.vertices.len() as bindings::types::GLint) / &object.stride_length)}
            }
        } else {
            panic!("{}", "Buffers were never intialized for object!");
        }
    })
}

impl Gl {
    pub fn draw_frame(&self, color: [f32; 4], scene: &Scene) {
        unsafe {
            self.inner.ClearColor(color[0], color[1], color[2], color[3]);
            self.inner.Clear(bindings::COLOR_BUFFER_BIT);
            render_scene(&self.inner, scene);
        }
    }
}
