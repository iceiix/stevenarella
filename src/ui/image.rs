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

ui_element!(Image {
    texture: render::Texture,
    width: f64,
    height: f64,

    t_x: f64,
    t_y: f64,
    t_width: f64,
    t_height: f64,

    r: u8,
    g: u8,
    b: u8,
    a: u8,
});

impl Image {
    base_impl!();

    pub fn new(texture: render::Texture,
               x: f64,
               y: f64,
               w: f64,
               h: f64,
               t_x: f64,
               t_y: f64,
               t_width: f64,
               t_height: f64,
               r: u8,
               g: u8,
               b: u8)
               -> Image {
        ui_create!(Image {
            texture: texture,
            x: x,
            y: y,
            width: w,
            height: h,

            t_x: t_x,
            t_y: t_y,
            t_width: t_width,
            t_height: t_height,

            r: r,
            g: g,
            b: b,
            a: 255,
        })
    }

    fn update(&mut self, _: &mut render::Renderer) {
    }

    fn draw(&mut self,
            renderer: &mut render::Renderer,
            r: &Region,
            width: f64,
            height: f64,
            _: f64)
            -> &Vec<u8> {
        if self.dirty {
            self.dirty = false;
            self.texture = renderer.check_texture(self.texture.clone());
            let mut e = render::ui::UIElement::new(&self.texture,
                                                   r.x,
                                                   r.y,
                                                   r.w,
                                                   r.h,
                                                   self.t_x,
                                                   self.t_y,
                                                   self.t_width,
                                                   self.t_height);
            e.r = self.r;
            e.g = self.g;
            e.b = self.b;
            e.a = self.a;
            e.layer = self.layer;
            self.data = e.bytes(width, height);
        }
        &self.data
    }

    pub fn get_texture(&self) -> render::Texture {
        self.texture.clone()
    }

    pub fn set_texture(&mut self, val: render::Texture) {
        self.texture = val;
        self.dirty = true;
    }

    lazy_field!(width, f64, get_width, set_width);
    lazy_field!(height, f64, get_height, set_height);

    lazy_field!(t_x, f64, get_t_x, set_t_x);
    lazy_field!(t_y, f64, get_t_y, set_t_y);
    lazy_field!(t_width, f64, get_t_width, set_t_width);
    lazy_field!(t_height, f64, get_t_height, set_t_height);

    lazy_field!(r, u8, get_r, set_r);
    lazy_field!(g, u8, get_g, set_g);
    lazy_field!(b, u8, get_b, set_b);
    lazy_field!(a, u8, get_a, set_a);
}

impl UIElement for Image {
    fn wrap(self) -> Element {
        Element::Image(self)
    }

    fn unwrap_ref<'a>(e: &'a Element) -> &'a Image {
        match e {
            &Element::Image(ref val) => val,
            _ => panic!("Incorrect type"),
        }
    }

    fn unwrap_ref_mut<'a>(e: &'a mut Element) -> &'a mut Image {
        match e {
            &mut Element::Image(ref mut val) => val,
            _ => panic!("Incorrect type"),
        }
    }

    fn get_attachment(&self) -> (VAttach, HAttach) {
        (self.v_attach, self.h_attach)
    }

    fn get_offset(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    fn get_size(&self) -> (f64, f64) {
        (self.width, self.height)
    }
}
