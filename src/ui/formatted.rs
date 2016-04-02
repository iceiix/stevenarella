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

ui_element!(Formatted {
    val: format::Component,
    width: f64,
    height: f64,
    scale_x: f64,
    scale_y: f64,

    text: Vec<Element>,
    max_width: f64,
    lines: usize,
});

impl Formatted {
    base_impl!();

    pub fn new(renderer: &mut render::Renderer,
               val: format::Component,
               x: f64,
               y: f64)
               -> Formatted {
        let mut f = ui_create!(Formatted {
            val: val,
            x: x,
            y: y,
            width: 0.0,
            height: 18.0,
            scale_x: 1.0,
            scale_y: 1.0,

            text: Vec::new(),
            max_width: -1.0,
            lines: 0,
        });
        f.init_component(renderer);
        f
    }

    pub fn with_width_limit(renderer: &mut render::Renderer,
                            val: format::Component,
                            x: f64,
                            y: f64,
                            max_width: f64)
                            -> Formatted {
        let mut f = ui_create!(Formatted {
            val: val,
            x: x,
            y: y,
            width: 0.0,
            height: 18.0,
            scale_x: 1.0,
            scale_y: 1.0,

            text: Vec::new(),
            max_width: max_width,
            lines: 0,
        });
        f.init_component(renderer);
        f
    }

    pub fn set_component(&mut self, renderer: &mut render::Renderer, val: format::Component) {
        self.val = val;
        self.init_component(renderer);
    }

    fn init_component(&mut self, renderer: &mut render::Renderer) {
        self.text.clear();
        let mut state = FormatState {
            lines: 0,
            width: 0.0,
            offset: 0.0,
            text: Vec::new(),
            max_width: self.max_width,
            renderer: &renderer,
        };
        state.build(&self.val, format::Color::White);
        self.height = (state.lines + 1) as f64 * 18.0;
        self.width = state.width;
        self.lines = state.lines;
        self.text = state.text;
        self.dirty = true;
    }

    fn update(&mut self, renderer: &mut render::Renderer) {
        self.init_component(renderer);
    }

    fn draw(&mut self,
            renderer: &mut render::Renderer,
            r: &Region,
            width: f64,
            height: f64,
            delta: f64)
            -> &Vec<u8> {
        if self.dirty {
            self.dirty = false;
            self.data.clear();
            let sx = r.w / self.width;
            let sy = r.h / self.height;

            for e in &mut self.text {
                let reg = e.get_draw_region(sx, sy, r);
                e.set_dirty(true);
                self.data.extend(e.draw(renderer, &reg, width, height, delta));
            }
        }
        &self.data
    }

    lazy_field!(width, f64, get_width, set_width);
    lazy_field!(height, f64, get_height, set_height);
    lazy_field!(scale_x, f64, get_scale_x, set_scale_x);
    lazy_field!(scale_y, f64, get_scale_y, set_scale_y);

}

impl UIElement for Formatted {
    fn wrap(self) -> Element {
        Element::Formatted(self)
    }

    fn unwrap_ref<'a>(e: &'a Element) -> &'a Formatted {
        match e {
            &Element::Formatted(ref val) => val,
            _ => panic!("Incorrect type"),
        }
    }

    fn unwrap_ref_mut<'a>(e: &'a mut Element) -> &'a mut Formatted {
        match e {
            &mut Element::Formatted(ref mut val) => val,
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

struct FormatState<'a> {
    max_width: f64,
    lines: usize,
    offset: f64,
    width: f64,
    text: Vec<Element>,
    renderer: &'a render::Renderer,
}

impl <'a> FormatState<'a> {
    fn build(&mut self, c: &format::Component, color: format::Color) {
        match c {
            &format::Component::Text(ref txt) => {
                let col = FormatState::get_color(&txt.modifier, color);
                self.append_text(&txt.text, col);
                let modi = &txt.modifier;
                if let Some(ref extra) = modi.extra {
                    for e in extra {
                        self.build(e, col);
                    }
                }
            }
        }
    }

    fn append_text(&mut self, txt: &str, color: format::Color) {
        let mut width = 0.0;
        let mut last = 0;
        for (i, c) in txt.char_indices() {
            let size = self.renderer.ui.size_of_char(c) + 2.0;
            if (self.max_width > 0.0 && self.offset + width + size > self.max_width) || c == '\n' {
                let (rr, gg, bb) = color.to_rgb();
                let text = Text::new(self.renderer,
                                     &txt[last..i],
                                     self.offset,
                                     (self.lines * 18 + 1) as f64,
                                     rr,
                                     gg,
                                     bb);
                self.text.push(text.wrap());
                last = i;
                if c == '\n' {
                    last += 1;
                }
                self.offset = 0.0;
                self.lines += 1;
                width = 0.0;
            }
            width += size;
            if self.offset + width > self.width {
                self.width = self.offset + width;
            }
        }

        if last != txt.len() {
            let (rr, gg, bb) = color.to_rgb();
            let text = Text::new(self.renderer,
                                 &txt[last..],
                                 self.offset,
                                 (self.lines * 18 + 1) as f64,
                                 rr,
                                 gg,
                                 bb);
            self.offset += text.width + 4.0; // TODO Why is this 4 not 2?
            self.text.push(text.wrap());
            if self.offset > self.width {
                self.width = self.offset;
            }
        }
    }

    fn get_color(modi: &format::Modifier, color: format::Color) -> format::Color {
        modi.color.unwrap_or(color)
    }
}
