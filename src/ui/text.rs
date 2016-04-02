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

ui_element!(Text {
    val: String,
    width: f64,
    height: f64,
    scale_x: f64,
    scale_y: f64,
    rotation: f64,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
});

impl Text {
    base_impl!();

    pub fn new(renderer: &render::Renderer,
               val: &str,
               x: f64,
               y: f64,
               r: u8,
               g: u8,
               b: u8)
               -> Text {
        ui_create!(Text {
            val: val.to_owned(),
            x: x,
            y: y,
            width: renderer.ui.size_of_string(val),
            height: 18.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            r: r,
            g: g,
            b: b,
            a: 255,
        })
    }

    fn update(&mut self, renderer: &mut render::Renderer) {
        self.width = renderer.ui.size_of_string(&self.val);
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
            let sx = r.w / self.width;
            let sy = r.h / self.height;
            let mut text = if self.rotation == 0.0 {
                renderer.ui.new_text_scaled(&self.val,
                                            r.x,
                                            r.y,
                                            sx * self.scale_x,
                                            sy * self.scale_y,
                                            self.r,
                                            self.g,
                                            self.b)
            } else {
                let c = self.rotation.cos();
                let s = self.rotation.sin();
                let tmpx = r.w / 2.0;
                let tmpy = r.h / 2.0;
                let w = (tmpx * c - tmpy * s).abs();
                let h = (tmpy * c + tmpx * s).abs();
                renderer.ui.new_text_rotated(&self.val,
                                             r.x + w - (r.w / 2.0),
                                             r.y + h - (r.h / 2.0),
                                             sx * self.scale_x,
                                             sy * self.scale_y,
                                             self.rotation,
                                             self.r,
                                             self.g,
                                             self.b)
            };
            for e in &mut text.elements {
                e.a = self.a;
                e.layer = self.layer;
            }
            self.data = text.bytes(width, height);
        }
        &self.data
    }

    pub fn get_text(&self) -> &str {
        &self.val
    }

    pub fn set_text(&mut self, renderer: &render::Renderer, val: &str) {
        self.dirty = true;
        self.val = val.to_owned();
        self.width = renderer.ui.size_of_string(val);
    }

    lazy_field!(width, f64, get_width, set_width);
    lazy_field!(height, f64, get_height, set_height);
    lazy_field!(scale_x, f64, get_scale_x, set_scale_x);
    lazy_field!(scale_y, f64, get_scale_y, set_scale_y);
    lazy_field!(rotation, f64, get_rotation, set_rotation);
    lazy_field!(r, u8, get_r, set_r);
    lazy_field!(g, u8, get_g, set_g);
    lazy_field!(b, u8, get_b, set_b);

}

impl UIElement for Text {
    fn wrap(self) -> Element {
        Element::Text(self)
    }

    fn unwrap_ref<'a>(e: &'a Element) -> &'a Text {
        match e {
            &Element::Text(ref val) => val,
            _ => panic!("Incorrect type"),
        }
    }

    fn unwrap_ref_mut<'a>(e: &'a mut Element) -> &'a mut Text {
        match e {
            &mut Element::Text(ref mut val) => val,
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
        ((self.width + 2.0) * self.scale_x, self.height * self.scale_y)
    }
}
