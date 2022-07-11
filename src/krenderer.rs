use crate::lib::kmath::*;
use crate::lib::kimg::*;

use glow::*;

// Stateful rendering
// even give it turtle graphics capabilities lol. I want to use more silly rotatey triangles and stuff
// can do even transformations and stuff
pub struct KRenderer {
    vbo: NativeBuffer,
    vao: NativeVertexArray,
    shader: NativeProgram,
    atlas: NativeTexture,
}

impl KRenderer {
    pub fn new(gl: &glow::Context, shader: NativeProgram, atlas: ImageBufferA) -> KRenderer {
        unsafe {
            let texture = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA as i32, atlas.w as i32, atlas.h as i32, 0, RGBA, glow::UNSIGNED_BYTE, Some(&atlas.bytes()));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.generate_mipmap(glow::TEXTURE_2D);
    
            // We construct a buffer and upload the data
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    
            // We now construct a vertex array to describe the format of the input buffer
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 4*4 + 4*3 + 4*2, 0);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, 4*4 + 4*3 + 4*2, 4*3);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 4*4 + 4*3 + 4*2, 4*4 + 4*3);
            gl.enable_vertex_attrib_array(2);
    
            KRenderer {
                vao,
                vbo,
                shader,
                atlas: texture,
            }
        }
    }

    pub fn send(&self, gl: &glow::Context, buf: &[u8]) {
        unsafe {
            gl.use_program(Some(self.shader));
            gl.bind_texture(glow::TEXTURE_2D, Some(self.atlas));
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, buf, glow::DYNAMIC_DRAW);
            let vert_count = buf.len() / (7*4);
            gl.draw_arrays(glow::TRIANGLES, 0, vert_count as i32);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.vbo);
            gl.delete_vertex_array(self.vao);
            gl.delete_texture(self.atlas);
        }
    }
}

pub struct KRCanvas {
    depth: f32,
    colour: Vec4,
    buf: Vec<u8>,
    uv_clip: Rect,
    uv_from: Rect,
    from_rect: Rect,
}

impl KRCanvas {
    pub fn new(screen_rect: Rect) -> KRCanvas {
        KRCanvas {
            depth: 1.0,
            colour: Vec4::new(0.0, 0.0, 0.0, 1.0), 
            buf: Vec::new(),
            uv_clip: Rect::new(0.0, 0.0, 1.0/20.0, 1.0/20.0),
            uv_from: Rect::new(-1000.0, -1000.0, 2000.0, 2000.0),
            from_rect: screen_rect,
        }
    }
    pub fn set_colour(&mut self, c: Vec4) {
        self.colour = c;
    }

    pub fn set_depth(&mut self, d: f32) {
        self.depth = d;
    }
    pub fn set_camera(&mut self, cam: Rect) {
        self.from_rect = cam;
    }
    pub fn triangle(&mut self, a: Vec2, b: Vec2, c: Vec2) {
        self.uv_from = Triangle{a,b,c}.aabb();
        let write_float_bytes = |buf: &mut Vec<u8>, x: f32| {
            for b in x.to_le_bytes() {
                buf.push(b);
            }
        };
        let write_vec2_bytes = |buf: &mut Vec<u8>, v: Vec2| {
            write_float_bytes(buf, v.x);
            write_float_bytes(buf, v.y);
        };
        let write_vec3_bytes = |buf: &mut Vec<u8>, v: Vec3| {
            write_float_bytes(buf, v.x);
            write_float_bytes(buf, v.y);
            write_float_bytes(buf, v.z);
        };
        let write_vec4_bytes = |buf: &mut Vec<u8>, v: Vec4| {
            write_float_bytes(buf, v.x);
            write_float_bytes(buf, v.y);
            write_float_bytes(buf, v.z);
            write_float_bytes(buf, v.w);
        };
        // ndc
        let to_rect = Rect::new(0.0, 0.0, 1.0, 1.0);

        // a
        write_vec3_bytes(&mut self.buf, a.transform(self.from_rect, to_rect).promote(self.depth));
        write_vec4_bytes(&mut self.buf, self.colour);
        write_vec2_bytes(&mut self.buf, a.transform(self.uv_from, self.uv_clip));
        // b
        write_vec3_bytes(&mut self.buf, b.transform(self.from_rect, to_rect).promote(self.depth));
        write_vec4_bytes(&mut self.buf, self.colour);
        write_vec2_bytes(&mut self.buf, b.transform(self.uv_from, self.uv_clip));
        // c
        write_vec3_bytes(&mut self.buf, c.transform(self.from_rect, to_rect).promote(self.depth));
        write_vec4_bytes(&mut self.buf, self.colour);
        write_vec2_bytes(&mut self.buf, c.transform(self.uv_from, self.uv_clip));
    }

    pub fn rect(&mut self, r: Rect) {
        self.uv_from = r;
        self.triangle(r.tl(), r.tr(), r.bl());
        self.triangle(r.bl(), r.tr(), r.br());
    }

    pub fn poly(&mut self, center: Vec2, radius: f32, n_sides: i32) {
        for i in 0..n_sides {
            let theta_1 = i as f32 * 2.0 * std::f32::consts::PI / n_sides as f32;
            let theta_2 = (i+1) as f32 * 2.0 * std::f32::consts::PI / n_sides as f32;
            self.triangle(center, center.offset_r_theta(radius, theta_1), center.offset_r_theta(radius, theta_2));
        }
    }
    pub fn poly_part(&mut self, center: Vec2, radius: f32, from_theta: f32, to_theta: f32, n_sides: i32) {
        for i in 0..n_sides {
            let theta_1 = from_theta + i as f32 * (to_theta - from_theta) / n_sides as f32;
            let theta_2 = from_theta + (i+1) as f32 * (to_theta - from_theta) / n_sides as f32;
            self.triangle(center, center.offset_r_theta(radius, theta_1), center.offset_r_theta(radius, theta_2));
        }
    }

    pub fn circle(&mut self, center: Vec2, radius: f32) {
        self.poly(center, radius, 40);
    }
    pub fn bytes(self) -> Vec<u8> {
        self.buf
    }
}