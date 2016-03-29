
use std::sync::{Arc, RwLock};
use std::f64::consts;
use ui;
use render;
use resources;
use time;
use rand;
use rand::Rng;

pub struct Logo {
    resources: Arc<RwLock<resources::Manager>>,

    shadow: ui::ElementRef<ui::Batch>,
    layer0: ui::ElementRef<ui::Batch>,

    text: ui::ElementRef<ui::Text>,
    text_base_scale: f64,
    text_orig_x: f64,
    text_index: isize,
    text_strings: Vec<String>,
}

impl Logo {
    pub fn new(resources: Arc<RwLock<resources::Manager>>,
               renderer: &mut render::Renderer,
               ui_container: &mut ui::Container)
               -> Logo {
        let mut l = Logo {
            resources: resources,
            shadow: Default::default(),
            layer0: Default::default(),
            text: Default::default(),
            text_base_scale: 0.0,
            text_orig_x: 0.0,
            text_index: -1,
            text_strings: Vec::new(),
        };
        l.init(renderer, ui_container);
        l
    }

    fn init(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let logo_str = {
            let res = self.resources.read().unwrap();
            let mut logo = res.open("steven", "logo/logo.txt").unwrap();
            let mut logo_str = String::new();
            logo.read_to_string(&mut logo_str).unwrap();
            logo_str
        };

        let solid = render::Renderer::get_texture(renderer.get_textures_ref(), "steven:solid");
        let front = render::Renderer::get_texture(renderer.get_textures_ref(), "blocks/planks_oak");

        let mut shadow_batch = ui::Batch::new(0.0, 8.0, 100.0, 100.0);
        let mut layer0 = ui::Batch::new(0.0, 8.0, 100.0, 100.0);

        shadow_batch.set_h_attach(ui::HAttach::Center);
        layer0.set_h_attach(ui::HAttach::Center);

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
                let mut shadow = ui::Image::new(solid.clone(),
                                                (x + 2) as f64,
                                                (y + 4) as f64,
                                                4.0,
                                                8.0,
                                                0.0,
                                                0.0,
                                                1.0,
                                                1.0,
                                                0,
                                                0,
                                                0);
                shadow.set_a(100);
                shadow_batch.add(shadow);


                let img = ui::Image::new(front.clone(),
                                         x as f64,
                                         y as f64,
                                         4.0,
                                         8.0,
                                         (x % 16) as f64 / 16.0,
                                         (y % 16) as f64 / 16.0,
                                         4.0 / 16.0,
                                         8.0 / 16.0,
                                         r,
                                         g,
                                         b);
                layer0.add(img);

                let width = (x + 4) as f64;
                if shadow_batch.get_width() < width {
                    shadow_batch.set_width(width);
                    layer0.set_width(width);
                }
            }
            row += 1;
        }
        {
            let res = self.resources.read().unwrap();
            let mut splashes = res.open_all("minecraft", "texts/splashes.txt");
            for file in &mut splashes {
                let mut texts = String::new();
                file.read_to_string(&mut texts).unwrap();
                for line in texts.lines() {
                    self.text_strings.push(line.to_owned().replace("\r", ""));
                }
            }
            let mut r: rand::XorShiftRng = rand::SeedableRng::from_seed([45, 64, 32, 12]);
            r.shuffle(&mut self.text_strings[..]);
        }

        shadow_batch.set_height(row as f64 * 8.0);
        layer0.set_height(row as f64 * 8.0);

        self.shadow = ui_container.add(shadow_batch);
        self.layer0 = ui_container.add(layer0);

        let mut txt = ui::Text::new(renderer, "", 0.0, -8.0, 255, 255, 0);
        txt.set_h_attach(ui::HAttach::Right);
        txt.set_v_attach(ui::VAttach::Bottom);
        txt.set_parent(&self.shadow);
        txt.set_rotation(-consts::PI / 8.0);

        let width = txt.get_width();
        self.text_base_scale = 300.0 / width;
        if self.text_base_scale > 1.0 {
            self.text_base_scale = 1.0;
        }
        txt.set_x((-width / 2.0) * self.text_base_scale);
        self.text_orig_x = txt.get_x();
        self.text = ui_container.add(txt);
    }

    pub fn tick(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let now = time::now().to_timespec();

        // Splash text
        let text = ui_container.get_mut(&self.text);
        let text_index = (now.sec / 15) as isize % self.text_strings.len() as isize;
        if self.text_index != text_index {
            self.text_index = text_index;
            text.set_text(renderer, &self.text_strings[text_index as usize]);
            let width = text.get_width();
            self.text_base_scale = 300.0 / width;
            if self.text_base_scale > 1.0 {
                self.text_base_scale = 1.0;
            }
            text.set_x((-width / 2.0) * self.text_base_scale);
            self.text_orig_x = text.get_x();
        }

        let timer = now.nsec as f64 / 1000000000.0;
        let mut offset = timer / 0.5;
        if offset > 1.0 {
            offset = 2.0 - offset;
        }
        offset = ((offset * consts::PI).cos() + 1.0) / 2.0;
        text.set_scale_x((0.7 + (offset / 3.0)) * self.text_base_scale);
        text.set_scale_y((0.7 + (offset / 3.0)) * self.text_base_scale);
        let scale = text.get_scale_x();
        text.set_x(self.text_orig_x * scale * self.text_base_scale);
    }

    pub fn remove(&self, ui_container: &mut ui::Container) {
        ui_container.remove(&self.shadow);
        ui_container.remove(&self.layer0);
        ui_container.remove(&self.text);
    }
}
