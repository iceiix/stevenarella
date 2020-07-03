use crate::render;
use crate::resources;
use crate::ui;
use rand::{self, seq::SliceRandom};
use std::f64::consts;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Logo {
    _shadow: ui::BatchRef,
    _layer0: ui::BatchRef,

    text: ui::TextRef,
    text_base_scale: f64,
    text_orig_x: f64,
    text_index: isize,
    text_strings: Vec<String>,
}

impl Logo {
    pub fn new(
        resources: Arc<RwLock<resources::Manager>>,
        ui_container: &mut ui::Container,
    ) -> Logo {
        let logo_str = {
            let res = resources.read().unwrap();
            let mut logo = res.open("steven", "logo/logo.txt").unwrap();
            let mut logo_str = String::new();
            logo.read_to_string(&mut logo_str).unwrap();
            logo_str
        };

        let shadow_batch = ui::BatchBuilder::new()
            .position(0.0, 8.0)
            .size(100.0, 100.0)
            .alignment(ui::VAttach::Top, ui::HAttach::Center)
            .create(ui_container);
        let layer0 = ui::BatchBuilder::new()
            .position(0.0, 8.0)
            .size(100.0, 100.0)
            .draw_index(1)
            .alignment(ui::VAttach::Top, ui::HAttach::Center)
            .create(ui_container);

        let mut row = 0;
        for line in logo_str.lines() {
            if line.is_empty() {
                continue;
            }
            for (i, c) in line.chars().enumerate() {
                if c == ' ' {
                    continue;
                }
                let x = i * 4;
                let y = row * 8;
                let (r, g, b) = if c == ':' {
                    (255, 255, 255)
                } else {
                    (170, 170, 170)
                };
                ui::ImageBuilder::new()
                    .texture("steven:solid")
                    .position((x + 2) as f64, (y + 4) as f64)
                    .size(4.0, 8.0)
                    .colour((0, 0, 0, 100))
                    .attach(&mut *shadow_batch.borrow_mut());

                ui::ImageBuilder::new()
                    .texture("minecraft:blocks/planks_oak")
                    .position(x as f64, y as f64)
                    .size(4.0, 8.0)
                    .texture_coords((
                        (x % 16) as f64 / 16.0,
                        (y % 16) as f64 / 16.0,
                        4.0 / 16.0,
                        8.0 / 16.0,
                    ))
                    .colour((r, g, b, 255))
                    .attach(&mut *layer0.borrow_mut());

                let width = (x + 4) as f64;
                if shadow_batch.borrow().width < width {
                    shadow_batch.borrow_mut().width = width;
                    layer0.borrow_mut().width = width;
                }
            }
            row += 1;
        }

        shadow_batch.borrow_mut().height = row as f64 * 8.0;
        layer0.borrow_mut().height = row as f64 * 8.0;

        let mut text_strings = vec![];
        {
            let res = resources.read().unwrap();
            let mut splashes = res.open_all("minecraft", "texts/splashes.txt");
            for file in &mut splashes {
                let mut texts = String::new();
                file.read_to_string(&mut texts).unwrap();
                for line in texts.lines() {
                    text_strings.push(line.to_owned().replace("\r", ""));
                }
            }
            let mut r: rand_pcg::Pcg32 =
                rand::SeedableRng::from_seed([45, 0, 0, 0, 64, 0, 0, 0, 32, 0, 0, 0, 12, 0, 0, 0]);
            text_strings.shuffle(&mut r);
        }

        let txt = ui::TextBuilder::new()
            .text("")
            .position(0.0, -8.0)
            .colour((255, 255, 0, 255))
            .rotation(-consts::PI / 8.0)
            .alignment(ui::VAttach::Bottom, ui::HAttach::Right)
            .draw_index(1)
            .create(&mut *layer0.borrow_mut());

        let width = txt.borrow().width;
        let mut text_base_scale = 300.0 / width;
        if text_base_scale > 1.0 {
            text_base_scale = 1.0;
        }
        txt.borrow_mut().x = (-width / 2.0) * text_base_scale;
        let text_orig_x = txt.borrow().x;

        Logo {
            _shadow: shadow_batch,
            _layer0: layer0,
            text: txt,
            text_base_scale,
            text_orig_x,
            text_index: -1,
            text_strings,
        }
    }

    pub fn tick(&mut self, renderer: &mut render::Renderer) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        // Splash text
        let text_index = (now.as_secs() / 15) as isize % self.text_strings.len() as isize;
        let mut text = self.text.borrow_mut();
        if self.text_index != text_index {
            self.text_index = text_index;
            text.text = self.text_strings[text_index as usize].clone();
            let width = (renderer.ui.size_of_string(&text.text) + 2.0) * text.scale_x;
            self.text_base_scale = 300.0 / width;
            if self.text_base_scale > 1.0 {
                self.text_base_scale = 1.0;
            }
            text.x = (-width / 2.0) * self.text_base_scale;
            self.text_orig_x = text.x;
        }

        let timer = now.subsec_nanos() as f64 / 1000000000.0;
        let mut offset = timer / 0.5;
        if offset > 1.0 {
            offset = 2.0 - offset;
        }
        offset = ((offset * consts::PI).cos() + 1.0) / 2.0;
        text.scale_x = (0.7 + (offset / 3.0)) * self.text_base_scale;
        text.scale_y = (0.7 + (offset / 3.0)) * self.text_base_scale;
        let scale = text.scale_x;
        text.x = self.text_orig_x * scale * self.text_base_scale;
    }
}
