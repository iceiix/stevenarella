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

ui_element!(TextBox {
    input: String,
    width: f64,
    height: f64,
    password: bool,
    button: Button,
    text: Text,
    cursor_tick: f64,
    was_focused: bool,
    submit_funcs: Vec<Rc<ClickFunc>>,
});

impl TextBox {
    base_impl!();

    pub fn new(renderer: &render::Renderer,
                input: &str,
                x: f64, y: f64, w: f64, h: f64
                ) -> TextBox {
        let mut btn = Button::new(0.0, 0.0, w, h);
        btn.set_disabled(true);
        let mut txt = Text::new(renderer, input, 5.0, 0.0, 255, 255, 255);
        txt.set_v_attach(VAttach::Middle);
        let mut tbox = ui_create!(TextBox {
            input: input.to_owned(),
            x: x,
            y: y,
            width: w,
            height: h,
            password: false,
            button: btn,
            text: txt,
            cursor_tick: 0.0,
            was_focused: false,
            submit_funcs: vec![],
        });
        tbox.can_focus = true;
        tbox
    }

    fn update(&mut self, renderer: &mut render::Renderer) {
        self.text.update(renderer);
    }

    fn draw(&mut self,
            renderer: &mut render::Renderer,
            r: &Region,
            width: f64,
            height: f64,
            delta: f64)
            -> &Vec<u8> {
        if self.dirty || self.focused || self.was_focused {
            self.was_focused = self.focused;
            self.data.clear();
            self.dirty = false;

            self.cursor_tick += delta;
            if self.cursor_tick > 3000.0 {
                self.cursor_tick -= 3000.0;
            }

            let mut txt = self.transform_input();
            if self.focused && ((self.cursor_tick / 30.0) as i32) % 2 == 0 {
                txt.push('|');
            }
            self.text.set_text(renderer, &txt);

            let sx = r.w / self.width;
            let sy = r.h / self.height;
            let reg = Container::get_draw_region_raw(&self.button, sx, sy, r);
            self.button.dirty = true;
            self.data.extend(self.button.draw(renderer, &reg, width, height, delta));

            let reg = Container::get_draw_region_raw(&self.text, sx, sy, r);
            self.text.dirty = true;
            self.data.extend(self.text.draw(renderer, &reg, width, height, delta));
        }
        &self.data
    }

    pub fn get_input(&self) -> String {
        self.input.clone()
    }

    pub fn set_input(&mut self, renderer: &render::Renderer, input: &str) {
        self.dirty = true;
        self.input = input.to_owned();
        let txt = self.transform_input();
        self.text.set_text(renderer, &txt);
    }

    pub fn add_submit_func<F: Fn(&mut ::Game, &mut Container) + 'static>(&mut self, f: F) {
        self.submit_funcs.push(Rc::new(f));
    }

    fn transform_input(&self) -> String {
        if self.password {
            ::std::iter::repeat('*').take(self.input.len()).collect()
        } else {
            self.input.clone()
        }
    }

    lazy_field!(width, f64, get_width, set_width);
    lazy_field!(height, f64, get_height, set_height);
    lazy_field!(password, bool, is_password, set_password);
}

impl UIElement for TextBox {

    fn key_press(&mut self, _game: &mut ::Game, key: Keycode, down: bool) -> Vec<Rc<ClickFunc>> {
        match (key, down) {
            (Keycode::Backspace, false) => {self.input.pop();},
            (Keycode::Return, false) => return self.submit_funcs.clone(),
            _ => {},
        }
        vec![]
    }

    fn key_type(&mut self, _game: &mut ::Game, c: char) -> Vec<Rc<ClickFunc>> {
        self.input.push(c);
        vec![]
    }

    fn wrap(self) -> Element {
        Element::TextBox(self)
    }

    fn unwrap_ref<'a>(e: &'a Element) -> &'a TextBox {
        match e {
            &Element::TextBox(ref val) => val,
            _ => panic!("Incorrect type"),
        }
    }

    fn unwrap_ref_mut<'a>(e: &'a mut Element) -> &'a mut TextBox {
        match e {
            &mut Element::TextBox(ref mut val) => val,
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
