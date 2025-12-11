use glow::{HasContext, PixelUnpackData};
use remdes::Region;

pub struct Texture2D {
    pub vao: glow::NativeVertexArray,
    pub _vbo: glow::NativeBuffer,
    pub tex: glow::NativeTexture,
    pub width: i32,
    pub height: i32,
}

impl Texture2D {
    pub fn new(gl: &glow::Context) -> Self {
        unsafe {
            // --- Full-screen quad (pos.xy, texcoord.xy) ---
            let vertices: [f32; 16] = [
                // pos      // texcoord
                -1.0, -1.0, 0.0, 0.0, // bottom-left
                1.0, -1.0, 1.0, 0.0, // bottom-right
                -1.0, 1.0, 0.0, 1.0, // top-left
                1.0, 1.0, 1.0, 1.0, // top-right
            ];

            // --- VAO/VBO setup ---
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&vertices),
                glow::STATIC_DRAW,
            );

            let stride = (4 * std::mem::size_of::<f32>()) as i32;

            // aPos (location = 0)
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);

            // aTexCoord (location = 1)
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 2 * 4);

            // --- Texture setup ---
            let tex = gl.create_texture().expect("Cannot create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);

            // --- Texture params ---
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );

            Self {
                vao,
                _vbo: vbo, // TODO - currently not modifying vertex data,
                tex,
                width: 0,
                height: 0,
            }
        }
    }

    pub fn update(&mut self, gl: &glow::Context, f: &Region) {
        unsafe {
            let [x, y, w, h] = [f.x(), f.y(), f.w(), f.h()];

            // (Re)allocate texture if the size changed or it's the first time
            if self.width != w || self.height != h {
                self.width = w;
                self.height = h;

                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA8 as i32,
                    w,
                    h,
                    0,
                    glow::BGRA,
                    glow::UNSIGNED_BYTE,
                    PixelUnpackData::Slice(Some(f.data())),
                );
            } else {
                // update only changed pixels
                gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    x,
                    y,
                    w,
                    h,
                    glow::BGRA,
                    glow::UNSIGNED_BYTE,
                    PixelUnpackData::Slice(Some(f.data())),
                );
            }
        }
    }

    pub fn delete(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_texture(self.tex);
            gl.delete_vertex_array(self.vao);
        }
    }
}
