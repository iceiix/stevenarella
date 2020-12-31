// Copyright 2016 Matthew Collins
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::gl;
use crate::render;
use crate::render::glsl;
use crate::render::shaders;
use crate::resources;
use byteorder::{NativeEndian, WriteBytesExt};
use image::GenericImageView;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

const UI_WIDTH: f64 = 854.0;
const UI_HEIGHT: f64 = 480.0;

pub struct UIState {
    textures: Arc<RwLock<render::TextureManager>>,
    resources: Arc<RwLock<resources::Manager>>,
    pub version: usize,

    data: Vec<u8>,
    prev_size: usize,
    count: usize,

    array: gl::VertexArray,
    buffer: gl::Buffer,
    index_buffer: gl::Buffer,
    index_type: gl::Type,
    max_index: usize,

    shader: UIShader,

    // Font
    font_pages: Vec<Option<render::Texture>>,
    font_character_info: Vec<(i32, i32)>,
    char_map: HashMap<char, char>,
    page_width: f64,
    page_height: f64,
}

init_shader! {
    Program UIShader {
        vert = "ui_vertex",
        frag = "ui_frag",
        attribute = {
            required position => "aPosition",
            required texture_info => "aTextureInfo",
            required texture_offset => "aTextureOffset",
            required color => "aColor",
        },
        uniform = {
            required texture => "textures",
            required screensize => "screenSize",
        },
    }
}

impl UIState {
    pub fn new(
        glsl: &glsl::Registry,
        textures: Arc<RwLock<render::TextureManager>>,
        res: Arc<RwLock<resources::Manager>>,
    ) -> UIState {
        let shader = UIShader::new(glsl);

        let array = gl::VertexArray::new();
        array.bind();
        let buffer = gl::Buffer::new();
        buffer.bind(gl::ARRAY_BUFFER);
        shader.position.enable();
        shader.texture_info.enable();
        shader.texture_offset.enable();
        shader.color.enable();
        shader.position.vertex_pointer_int(3, gl::SHORT, 28, 0);
        shader
            .texture_info
            .vertex_pointer(4, gl::UNSIGNED_SHORT, false, 28, 8);
        shader
            .texture_offset
            .vertex_pointer_int(3, gl::SHORT, 28, 16);
        shader
            .color
            .vertex_pointer(4, gl::UNSIGNED_BYTE, true, 28, 24);

        let index_buffer = gl::Buffer::new();
        index_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);

        let mut pages = Vec::with_capacity(0x100);
        for _ in 0..0x100 {
            pages.push(Option::None);
        }

        let mut char_map = HashMap::new();
        let ascii_chars = "ÀÁÂÈÊËÍÓÔÕÚßãõğİıŒœŞşŴŵžȇ        \
                           !\"#$%&'()*+,-./0123456789:;\
                           <=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ \
                           ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜø£Ø×ƒáíóúñÑªº¿®¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞\
                           ╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αβΓπΣσμτΦΘΩδ∞∅∈∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■";
        for (pos, c) in ascii_chars.chars().enumerate() {
            char_map.insert(c, ::std::char::from_u32(pos as u32).unwrap());
        }

        let mut state = UIState {
            textures,
            resources: res,
            version: 0xFFFF,

            data: Vec::new(),
            count: 0,
            prev_size: 0,

            index_type: gl::UNSIGNED_BYTE,
            array,
            buffer,
            index_buffer,
            max_index: 0,

            shader,

            // Font
            font_pages: pages,
            font_character_info: vec![(0, 0); 0x10000],
            char_map,
            page_width: 0.0,
            page_height: 0.0,
        };
        state.load_font();
        state
    }

    pub fn tick(&mut self, width: u32, height: u32) {
        {
            let version = self.resources.read().unwrap().version();
            if self.version != version {
                self.version = version;
                self.load_font();
            }
        }
        // Prevent clipping with the world
        gl::clear(gl::ClearFlags::Depth);
        gl::enable(gl::BLEND);
        gl::blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        self.shader.program.use_program();
        self.shader.texture.set_int(0);
        if self.count > 0 {
            self.array.bind();
            if self.max_index < self.count {
                let (data, ty) = render::generate_element_buffer(self.count);
                self.index_type = ty;
                self.index_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);
                self.index_buffer
                    .set_data(gl::ELEMENT_ARRAY_BUFFER, &data, gl::DYNAMIC_DRAW);
                self.max_index = self.count;
            }

            self.shader
                .screensize
                .set_float2(width as f32, height as f32);

            self.buffer.bind(gl::ARRAY_BUFFER);
            self.index_buffer.bind(gl::ELEMENT_ARRAY_BUFFER);
            if self.data.len() > self.prev_size {
                self.prev_size = self.data.len();
                self.buffer
                    .set_data(gl::ARRAY_BUFFER, &self.data, gl::STREAM_DRAW);
            } else {
                self.buffer.re_set_data(gl::ARRAY_BUFFER, &self.data);
            }
            gl::draw_elements(gl::TRIANGLES, self.count as i32, self.index_type, 0);
        }

        gl::disable(gl::BLEND);
        self.data.clear();
        self.count = 0;
    }

    pub fn add_bytes(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
        self.count += (data.len() / (28 * 4)) * 6;
    }

    pub fn character_texture(&mut self, c: char) -> render::Texture {
        let raw = c as u32;
        let page = raw >> 8;
        // Lazy load fonts to size memory
        if self.font_pages[page as usize].is_none() {
            let name = if page == 0 {
                "font/ascii".to_owned()
            } else {
                format!("font/unicode_page_{:02X}", page)
            };
            let textures = self.textures.clone();
            self.font_pages[page as usize] = Some(render::Renderer::get_texture(&textures, &name));
        }
        let p = self.font_pages[page as usize].clone().unwrap();

        let raw = if page == 0 {
            (*self.char_map.get(&c).unwrap_or(&c)) as u32
        } else {
            raw
        };
        let ch = raw & 0xFF;
        let cx = ch & 0xF;
        let cy = ch >> 4;
        let info = self.font_character_info[raw as usize];
        if page == 0 {
            let sw = (self.page_width / 16.0) as u32;
            let sh = (self.page_height / 16.0) as u32;
            return p.relative(
                (cx * sw + info.0 as u32) as f32 / (self.page_width as f32),
                (cy * sh) as f32 / (self.page_height as f32),
                (info.1 - info.0) as f32 / (self.page_width as f32),
                (sh as f32) / (self.page_height as f32),
            );
        }
        p.relative(
            (cx * 16 + info.0 as u32) as f32 / 256.0,
            (cy * 16) as f32 / 256.0,
            (info.1 - info.0) as f32 / 256.0,
            16.0 / 256.0,
        )
    }

    pub fn size_of_string(&self, val: &str) -> f64 {
        let mut size = 0.0;
        for c in val.chars() {
            size += self.size_of_char(c) + 2.0;
        }
        size - 2.0
    }

    pub fn size_of_char(&self, c: char) -> f64 {
        if c == ' ' {
            return 4.0;
        }
        let r = c as u32;
        if r >> 8 == 0 {
            let r = (*self.char_map.get(&c).unwrap_or(&c)) as u32;
            let info = self.font_character_info[r as usize];
            let sw = self.page_width / 16.0;
            return (((info.1 - info.0) as f64) / sw) * 16.0;
        }
        let info = self.font_character_info[c as usize];
        (info.1 - info.0) as f64
    }

    fn load_font(&mut self) {
        for page in &mut self.font_pages {
            *page = None;
        }
        let res = self.resources.read().unwrap();
        if let Some(mut info) = res.open("minecraft", "font/glyph_sizes.bin") {
            let mut data = Vec::with_capacity(0x10000);
            info.read_to_end(&mut data).unwrap();
            for (i, info) in self.font_character_info.iter_mut().enumerate() {
                // Top nibble - start position
                // Bottom nibble - end position
                info.0 = (data[i] >> 4) as i32;
                info.1 = (data[i] & 0xF) as i32 + 1;
            }
        }
        if let Some(mut val) = res.open("minecraft", "textures/font/ascii.png") {
            let mut data = Vec::new();
            val.read_to_end(&mut data).unwrap();
            if let Ok(img) = image::load_from_memory(&data) {
                let (width, height) = img.dimensions();
                self.page_width = width as f64;
                self.page_height = height as f64;
                let sw = width / 16;
                let sh = height / 16;
                for i in 0..256 {
                    let cx = (i & 0xF) * sw;
                    let cy = (i >> 4) * sh;
                    let mut start = true;
                    'x_loop: for x in 0..sw {
                        for y in 0..sh {
                            let a = img.get_pixel(cx + x, cy + y).0[3];
                            if start && a != 0 {
                                self.font_character_info[i as usize].0 = x as i32;
                                start = false;
                                continue 'x_loop;
                            } else if !start && a != 0 {
                                continue 'x_loop;
                            }
                        }
                        if !start {
                            self.font_character_info[i as usize].1 = x as i32;
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn new_text(&mut self, val: &str, x: f64, y: f64, r: u8, g: u8, b: u8) -> UIText {
        self.new_text_scaled(val, x, y, 1.0, 1.0, r, g, b)
    }

    pub fn new_text_scaled(
        &mut self,
        val: &str,
        x: f64,
        y: f64,
        sx: f64,
        sy: f64,
        r: u8,
        g: u8,
        b: u8,
    ) -> UIText {
        self.create_text(val, x, y, sx, sy, 0.0, r, g, b)
    }

    pub fn new_text_rotated(
        &mut self,
        val: &str,
        x: f64,
        y: f64,
        sx: f64,
        sy: f64,
        rotation: f64,
        r: u8,
        g: u8,
        b: u8,
    ) -> UIText {
        self.create_text(val, x, y, sx, sy, rotation, r, g, b)
    }

    fn create_text(
        &mut self,
        val: &str,
        x: f64,
        y: f64,
        sx: f64,
        sy: f64,
        rotation: f64,
        r: u8,
        g: u8,
        b: u8,
    ) -> UIText {
        let mut elements = Vec::new();
        let mut offset = 0.0;
        for ch in val.chars() {
            if ch == ' ' {
                offset += 6.0;
                continue;
            }
            let texture = self.character_texture(ch);
            let w = self.size_of_char(ch);

            let mut dsx = offset + 2.0;
            let mut dsy = 2.0;
            let mut dx = offset;
            let mut dy = 0.0;
            if rotation != 0.0 {
                let c = rotation.cos();
                let s = rotation.sin();
                let tmpx = dsx - (w * 0.5);
                let tmpy = dsy - (16.0 * 0.5);
                dsx = (w * 0.5) + (tmpx * c - tmpy * s);
                dsy = (16.0 * 0.5) + (tmpy * c + tmpx * s);
                let tmpx = dx - (w * 0.5);
                let tmpy = dy - (16.0 * 0.5);
                dx = (w * 0.5) + (tmpx * c - tmpy * s);
                dy = (16.0 * 0.5) + (tmpy * c + tmpx * s);
            }

            let mut shadow = UIElement::new(
                &texture,
                x + dsx * sx,
                y + dsy * sy,
                w * sx,
                16.0 * sy,
                0.0,
                0.0,
                1.0,
                1.0,
            );
            shadow.r = ((r as f64) * 0.25) as u8;
            shadow.g = ((g as f64) * 0.25) as u8;
            shadow.b = ((b as f64) * 0.25) as u8;
            shadow.rotation = rotation;
            elements.push(shadow);

            let mut text = UIElement::new(
                &texture,
                x + dx * sx,
                y + dy * sy,
                w * sx,
                16.0 * sy,
                0.0,
                0.0,
                1.0,
                1.0,
            );
            text.r = r;
            text.g = g;
            text.b = b;
            text.rotation = rotation;
            elements.push(text);
            offset += w + 2.0;
        }
        UIText {
            elements,
            width: (offset - 2.0) * sx,
        }
    }
}

pub struct UIText {
    pub elements: Vec<UIElement>,
    pub width: f64,
}

impl UIText {
    pub fn bytes(&self, width: f64, height: f64) -> Vec<u8> {
        let mut buf = Vec::with_capacity(28 * 4 * self.elements.len());
        for e in &self.elements {
            buf.extend(e.bytes(width, height));
        }
        buf
    }
}

pub struct UIElement {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub layer: isize,
    pub t_x: u16,
    pub t_y: u16,
    pub t_w: u16,
    pub t_h: u16,
    pub t_offsetx: i16,
    pub t_offsety: i16,
    pub t_atlas: i16,
    pub t_sizew: i16,
    pub t_sizeh: i16,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    pub rotation: f64,
}

impl UIElement {
    pub fn new(
        tex: &render::Texture,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        tx: f64,
        ty: f64,
        tw: f64,
        th: f64,
    ) -> UIElement {
        let twidth = tex.get_width();
        let theight = tex.get_height();
        UIElement {
            x: x / UI_WIDTH,
            y: y / UI_HEIGHT,
            w: width / UI_WIDTH,
            h: height / UI_HEIGHT,
            layer: 0,
            t_x: tex.get_x() as u16,
            t_y: tex.get_y() as u16,
            t_w: twidth as u16,
            t_h: theight as u16,
            t_atlas: tex.atlas as i16,
            t_offsetx: (tx * (twidth as f64) * 16.0) as i16,
            t_offsety: (ty * (theight as f64) * 16.0) as i16,
            t_sizew: (tw * (twidth as f64) * 16.0) as i16,
            t_sizeh: (th * (theight as f64) * 16.0) as i16,
            r: 255,
            g: 255,
            b: 255,
            a: 255,
            rotation: 0.0,
        }
    }

    pub fn bytes(&self, width: f64, height: f64) -> Vec<u8> {
        let mut buf = Vec::with_capacity(28 * 4);
        self.append_vertex(
            &mut buf,
            self.x,
            self.y,
            self.t_offsetx,
            self.t_offsety,
            width,
            height,
        );
        self.append_vertex(
            &mut buf,
            self.x + self.w,
            self.y,
            self.t_offsetx + self.t_sizew,
            self.t_offsety,
            width,
            height,
        );
        self.append_vertex(
            &mut buf,
            self.x,
            self.y + self.h,
            self.t_offsetx,
            self.t_offsety + self.t_sizeh,
            width,
            height,
        );
        self.append_vertex(
            &mut buf,
            self.x + self.w,
            self.y + self.h,
            self.t_offsetx + self.t_sizew,
            self.t_offsety + self.t_sizeh,
            width,
            height,
        );
        buf
    }

    #[allow(unused_must_use)]
    pub fn append_vertex(
        &self,
        buf: &mut Vec<u8>,
        x: f64,
        y: f64,
        tx: i16,
        ty: i16,
        width: f64,
        height: f64,
    ) {
        let mut dx = x as f64;
        let mut dy = y as f64;
        if self.rotation != 0.0 {
            let c = self.rotation.cos();
            let s = self.rotation.sin();
            let tmpx = dx - self.x - (self.w / 2.0);
            let tmpy = dy - self.y - (self.h / 2.0);
            dx = (self.w / 2.0) + (tmpx * c - tmpy * s) + self.x;
            dy = (self.h / 2.0) + (tmpy * c + tmpx * s) + self.y;
        }

        buf.write_i16::<NativeEndian>((dx * width + 0.5).floor() as i16);
        buf.write_i16::<NativeEndian>((dy * height + 0.5).floor() as i16);
        buf.write_i16::<NativeEndian>((self.layer * 256) as i16);
        buf.write_i16::<NativeEndian>(0);
        buf.write_u16::<NativeEndian>(self.t_x);
        buf.write_u16::<NativeEndian>(self.t_y);
        buf.write_u16::<NativeEndian>(self.t_w);
        buf.write_u16::<NativeEndian>(self.t_h);
        buf.write_i16::<NativeEndian>(tx);
        buf.write_i16::<NativeEndian>(ty);
        buf.write_i16::<NativeEndian>(self.t_atlas);
        buf.write_i16::<NativeEndian>(0);
        buf.write_u8(self.r);
        buf.write_u8(self.g);
        buf.write_u8(self.b);
        buf.write_u8(self.a);
    }
}
