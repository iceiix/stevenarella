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

ui_element!(Button {
    width: f64,
    height: f64,
    disabled: bool,
});

impl Button {
    base_impl!();

    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Button {
        let mut btn = ui_create!(Button {
            x: x,
            y: y,
            width: w,
            height: h,
            disabled: false,
        });
        btn.add_hover_func(|_,_,_|{}); // Force hover events to be called
        btn
    }

    fn update(&mut self, _renderer: &mut render::Renderer) {

    }

    fn draw(&mut self,
            renderer: &mut render::Renderer,
            r: &Region,
            width: f64,
            height: f64,
            _delta: f64)
            -> &Vec<u8> {
        if self.dirty {
            self.dirty = false;
            let sx = r.w / self.width;
            let sy = r.h / self.height;

            let offset = match (self.disabled, self.hovered) {
                    (true, _) => 46.0,
                    (false, true) => 86.0,
                    (false, false) => 66.0,
            };
            let texture = render::Renderer::get_texture(renderer.get_textures_ref(), "gui/widgets")
                      .relative(0.0, offset / 256.0, 200.0 / 256.0, 20.0 / 256.0);
            self.data.clear();

            self.data.extend(render::ui::UIElement::new(&texture, r.x, r.y, 4.0 * sx, 4.0 * sy, 0.0, 0.0, 2.0/200.0, 2.0/20.0).bytes(width, height));
            self.data.extend(render::ui::UIElement::new(&texture, r.x + r.w - 4.0 * sx, r.y, 4.0 * sx, 4.0 * sy, 198.0/200.0, 0.0, 2.0/200.0, 2.0/20.0).bytes(width, height));
            self.data.extend(render::ui::UIElement::new(&texture, r.x, r.y + r.h - 6.0 * sy, 4.0 * sx, 6.0 * sy, 0.0, 17.0/20.0, 2.0/200.0, 3.0/20.0).bytes(width, height));
            self.data.extend(render::ui::UIElement::new(&texture, r.x + r.w - 4.0 * sx, r.y + r.h - 6.0 * sy, 4.0 * sx, 6.0 * sy, 198.0/200.0, 17.0/20.0, 2.0/200.0, 3.0/20.0).bytes(width, height));

            let w = ((r.w / sx)/2.0) - 4.0;
            self.data.extend(render::ui::UIElement::new(
                &texture.relative(2.0/200.0, 0.0, 196.0/200.0, 2.0/20.0),
                r.x+4.0*sx, r.y, r.w - 8.0 * sx, 4.0 * sy, 0.0, 0.0, w/196.0, 1.0).bytes(width, height)
            );
            self.data.extend(render::ui::UIElement::new(
                &texture.relative(2.0/200.0, 17.0/20.0, 196.0/200.0, 3.0/20.0),
                r.x+4.0*sx, r.y+r.h-6.0*sy, r.w - 8.0 * sx, 6.0 * sy, 0.0, 0.0, w/196.0, 1.0).bytes(width, height)
            );

            let h = ((r.h / sy)/2.0) - 5.0;
            self.data.extend(render::ui::UIElement::new(
                &texture.relative(0.0/200.0, 2.0/20.0, 2.0/200.0, 15.0/20.0),
                r.x, r.y + 4.0*sy, 4.0 * sx, r.h - 10.0*sy, 0.0, 0.0, 1.0, h/16.0).bytes(width, height)
            );
            self.data.extend(render::ui::UIElement::new(
                &texture.relative(198.0/200.0, 2.0/20.0, 2.0/200.0, 15.0/20.0),
                r.x+r.w - 4.0 * sx, r.y + 4.0*sy, 4.0 * sx, r.h - 10.0*sy, 0.0, 0.0, 1.0, h/16.0).bytes(width, height)
            );


            self.data.extend(render::ui::UIElement::new(
                &texture.relative(2.0/200.0, 2.0/20.0, 196.0/200.0, 15.0/20.0),
                r.x+4.0*sx, r.y+4.0*sy, r.w - 8.0 * sx, r.h - 10.0 * sy, 0.0, 0.0, w/196.0, h/16.0).bytes(width, height)
            );
        }
        &self.data
    }

    lazy_field!(width, f64, get_width, set_width);
    lazy_field!(height, f64, get_height, set_height);
    lazy_field!(disabled, bool, is_disabled, set_disabled);

}

impl UIElement for Button {
    fn wrap(self) -> Element {
        Element::Button(self)
    }

    fn unwrap_ref<'a>(e: &'a Element) -> &'a Button {
        match e {
            &Element::Button(ref val) => val,
            _ => panic!("Incorrect type"),
        }
    }

    fn unwrap_ref_mut<'a>(e: &'a mut Element) -> &'a mut Button {
        match e {
            &mut Element::Button(ref mut val) => val,
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
