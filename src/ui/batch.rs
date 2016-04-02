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

#[derive(Clone, Copy)]
pub struct BatchRef<T: UIElement> {
    index: usize,
    ty: PhantomData<T>,
}

ui_element!(Batch {
    width: f64,
    height: f64,

    elements: Vec<Element>,
});

impl Batch {
    base_impl!();

    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Batch {
        ui_create!(Batch {
            x: x,
            y: y,
            width: w,
            height: h,

            elements: Vec::new(),
        })
    }

    fn update(&mut self, _: &mut render::Renderer) {
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

            for e in &mut self.elements {
                let reg = e.get_draw_region(sx, sy, r);
                e.set_dirty(true);
                self.data.extend(e.draw(renderer, &reg, width, height, delta));
            }
        }
        &self.data
    }

    pub fn add<T: UIElement>(&mut self, e: T) -> BatchRef<T> {
        self.elements.push(e.wrap());
        BatchRef {
            index: self.elements.len() - 1,
            ty: PhantomData,
        }
    }

    pub fn get<T: UIElement>(&self, r: BatchRef<T>) -> &T {
        T::unwrap_ref(&self.elements[r.index])
    }

    pub fn get_mut<T: UIElement>(&mut self, r: BatchRef<T>) -> &mut T {
        self.dirty = true;
        T::unwrap_ref_mut(&mut self.elements[r.index])
    }

    pub fn get_mut_at<T: UIElement>(&mut self, index: usize) -> &mut T {
        self.dirty = true;
        T::unwrap_ref_mut(&mut self.elements[index])
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    lazy_field!(width, f64, get_width, set_width);
    lazy_field!(height, f64, get_height, set_height);
}

impl UIElement for Batch {
    fn wrap(self) -> Element {
        Element::Batch(self)
    }

    fn unwrap_ref<'a>(e: &'a Element) -> &'a Batch {
        match e {
            &Element::Batch(ref val) => val,
            _ => panic!("Incorrect type"),
        }
    }

    fn unwrap_ref_mut<'a>(e: &'a mut Element) -> &'a mut Batch {
        match e {
            &mut Element::Batch(ref mut val) => val,
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
