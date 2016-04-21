use console;
use render;
use ui;
use settings;

use std::rc::Rc;

pub fn new_submenu_button(text: &str, renderer: &mut render::Renderer, ui_container: &mut ui::Container, x: f64, y: f64) -> (ui::ElementRef<ui::Button>, ui::ElementRef<ui::Text>) {
    let (mut btn, mut txt) = super::new_button_text(renderer, text, x, y, 300.0, 40.0);
    btn.set_v_attach(ui::VAttach::Middle);
    btn.set_h_attach(ui::HAttach::Center);
    let ui_btn = ui_container.add(btn);
    txt.set_parent(&ui_btn);
    (ui_btn, ui_container.add(txt))
}

pub fn new_centered_button(text: &str, renderer: &mut render::Renderer, ui_container: &mut ui::Container, y: f64, vertical_attach: ui::VAttach) -> (ui::ElementRef<ui::Button>, ui::ElementRef<ui::Text>) {
    let (mut btn, mut txt) = super::new_button_text(renderer, text, 0.0, y, 400.0, 40.0);
    btn.set_v_attach(vertical_attach);
    btn.set_h_attach(ui::HAttach::Center);
    let ui_btn = ui_container.add(btn);
    txt.set_parent(&ui_btn);
    (ui_btn, ui_container.add(txt))
}

macro_rules! get_bool_str {
    ($fmt:expr, $val:expr, $val_true:expr, $val_false:expr) => (format!($fmt, if $val {
        $val_true
    } else {
        $val_false
    }).as_ref());
    ($fmt:expr, $val:expr) => (get_bool_string!($fmt, $val, "true", "false"));
}

macro_rules! get_matched_str {
    ($fmt:expr, $val:expr, $($to_match:expr => $result:expr),*) => (
        format!($fmt, match $val {
            $($to_match => $result.to_owned(), )*
            _ => $val.to_string(),
        }).as_ref()
    )
}

pub struct UIElements {
    elements: ui::Collection,
    background: ui::ElementRef<ui::Image>,
}

pub struct SettingsMenu {
    vars: Rc<console::Vars>,
    elements: Option<UIElements>,
    show_disconnect_button: bool
}

impl SettingsMenu {
    pub fn new(vars: Rc<console::Vars>, show_disconnect_button: bool) -> SettingsMenu {
        SettingsMenu {
            vars: vars,
            elements: None,
            show_disconnect_button: show_disconnect_button
        }
    }
}

impl super::Screen for SettingsMenu {
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let mut elements = ui::Collection::new();

        let mut background = ui::Image::new(
            render::Renderer::get_texture(renderer.get_textures_ref(), "steven:solid"),
            0.0, 0.0, 854.0, 480.0,
            0.0, 0.0, 1.0, 1.0,
            0, 0, 0
        );
        background.set_a(100);
        let background = elements.add(ui_container.add(background));

        // From top and down
        let (btn_audio_settings, txt_audio_settings) = new_submenu_button("Audio settings...", renderer, ui_container, -160.0, -50.0);
        super::button_action(ui_container, btn_audio_settings.clone(), Some(txt_audio_settings.clone()), move |game, _| {
            game.screen_sys.add_screen(Box::new(AudioSettingsMenu::new(game.vars.clone())));
        });
        elements.add(btn_audio_settings);
        elements.add(txt_audio_settings);

        let (btn_video_settings, txt_video_settings) = new_submenu_button("Video settings...", renderer, ui_container, 160.0, -50.0);
        super::button_action(ui_container, btn_video_settings.clone(), Some(txt_video_settings.clone()), move |game, _| {
            game.screen_sys.add_screen(Box::new(VideoSettingsMenu::new(game.vars.clone())));
        });
        elements.add(btn_video_settings);
        elements.add(txt_video_settings);

        let (btn_controls_settings, txt_controls_settings) = new_submenu_button("Controls...", renderer, ui_container, 160.0, 0.0);
        super::button_action(ui_container, btn_controls_settings.clone(), Some(txt_controls_settings.clone()), move |_, _| {
            // TODO: Implement this...
        });
        elements.add(btn_controls_settings);
        elements.add(txt_controls_settings);

        let (btn_locale_settings, txt_locale_settings) = new_submenu_button("Language...", renderer, ui_container, -160.0, 0.0);
        super::button_action(ui_container, btn_locale_settings.clone(), Some(txt_locale_settings.clone()), move |_, _| {
            // TODO: Implement this...
        });
        elements.add(btn_locale_settings);
        elements.add(txt_locale_settings);

        // Center bottom items
        let (btn_back_to_game, txt_back_to_game) = new_centered_button("Done", renderer, ui_container, 50.0, ui::VAttach::Bottom);
        super::button_action(ui_container, btn_back_to_game.clone(), Some(txt_back_to_game.clone()), move |game, _| {
            game.screen_sys.pop_screen();
            game.focused = true;
        });
        elements.add(btn_back_to_game);
        elements.add(txt_back_to_game);

        if self.show_disconnect_button {
            let (btn_exit_game, txt_exit_game) = new_centered_button("Disconnect", renderer, ui_container, 100.0, ui::VAttach::Bottom);
            super::button_action(ui_container, btn_exit_game.clone(), Some(txt_exit_game.clone()), move |game, _| {
                game.server.disconnect(None);
                game.screen_sys.replace_screen(Box::new(super::ServerList::new(None)));
            });
            elements.add(btn_exit_game);
            elements.add(txt_exit_game);
        }

        self.elements = Some(UIElements {
            elements: elements,
            background: background,
        });

    }
    fn on_deactive(&mut self, _renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        {
            let elements = self.elements.as_mut().unwrap();
            elements.elements.remove_all(ui_container);
        }
        self.elements = None;
    }

    // Called every frame the screen is active
    fn tick(&mut self, _delta: f64, renderer: &mut render::Renderer, ui_container: &mut ui::Container) -> Option<Box<super::Screen>> {
        let elements = self.elements.as_mut().unwrap();
        {
            let mode = ui_container.mode;
            let background = ui_container.get_mut(&elements.background);
            background.set_width(match mode {
                ui::Mode::Unscaled(scale) => 854.0 / scale,
                ui::Mode::Scaled => renderer.width as f64,
            });
            background.set_height(match mode {
                ui::Mode::Unscaled(scale) => 480.0 / scale,
                ui::Mode::Scaled => renderer.height as f64,
            });
        }
        None
    }

    // Events
    fn on_scroll(&mut self, _x: f64, _y: f64) {

    }

    fn is_closable(&self) -> bool {
        true
    }
}

pub struct VideoSettingsMenu {
    vars: Rc<console::Vars>,
    elements: Option<UIElements>,
}

impl VideoSettingsMenu {
    pub fn new(vars: Rc<console::Vars>) -> VideoSettingsMenu {
        VideoSettingsMenu {
            vars: vars,
            elements: None,
        }
    }
}

impl super::Screen for VideoSettingsMenu {
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let mut elements = ui::Collection::new();

        let mut background = ui::Image::new(
            render::Renderer::get_texture(renderer.get_textures_ref(), "steven:solid"),
            0.0, 0.0, 854.0, 480.0,
            0.0, 0.0, 1.0, 1.0,
            0, 0, 0
        );
        background.set_a(100);
        let background = elements.add(ui_container.add(background));

        // Load defaults
        let r_max_fps = *self.vars.get(settings::R_MAX_FPS);
        let r_fov = *self.vars.get(settings::R_FOV);
        let r_vsync = *self.vars.get(settings::R_VSYNC);

        // Setting buttons
        // TODO: Slider
        let (btn_fov, txt_fov) = new_submenu_button(get_matched_str!("FOV: {}", r_fov, 90 => "Normal", 110 => "Quake pro"), renderer, ui_container, -160.0, -50.0);
        elements.add(btn_fov);
        elements.add(txt_fov);

        let (btn_vsync, txt_vsync) = new_submenu_button(get_bool_str!("VSync: {}", r_vsync, "Enabled", "Disabled"), renderer, ui_container, -160.0, 0.0);
        elements.add(txt_vsync.clone());
        super::button_action(ui_container, btn_vsync.clone(), Some(txt_vsync.clone()), move | game, ui_container | {
            let r_vsync = !*game.vars.get(settings::R_VSYNC);
            let txt_vsync = ui_container.get_mut(&txt_vsync);
            txt_vsync.set_text(&game.renderer, get_bool_str!("VSync: {}", r_vsync, "Enabled", "Disabled"));
            game.vars.set(settings::R_VSYNC, r_vsync);
        });
        elements.add(btn_vsync);

        // TODO: Slider
        let (btn_fps_cap, txt_fps_cap) = new_submenu_button(get_matched_str!("FPS cap: {}", r_max_fps, 0 => "Unlimited", 15 => "Potato"), renderer, ui_container, 160.0, 0.0);
        elements.add(btn_fps_cap);
        elements.add(txt_fps_cap);

        let (btn_done, txt_done) = new_centered_button("Done", renderer, ui_container, 50.0, ui::VAttach::Bottom);
        super::button_action(ui_container, btn_done.clone(), Some(txt_done.clone()), move | game, _ | {
            game.screen_sys.pop_screen();
        });
        elements.add(btn_done);
        elements.add(txt_done);
        self.elements = Some(UIElements {
            elements: elements,
            background: background,
        });

    }
    fn on_deactive(&mut self, _renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        {
            let elements = self.elements.as_mut().unwrap();
            elements.elements.remove_all(ui_container);
        }
        self.elements = None;
    }

    // Called every frame the screen is active
    fn tick(&mut self, _delta: f64, renderer: &mut render::Renderer, ui_container: &mut ui::Container) -> Option<Box<super::Screen>> {
        let elements = self.elements.as_mut().unwrap();
        {
            let mode = ui_container.mode;
            let background = ui_container.get_mut(&elements.background);
            background.set_width(match mode {
                ui::Mode::Unscaled(scale) => 854.0 / scale,
                ui::Mode::Scaled => renderer.width as f64,
            });
            background.set_height(match mode {
                ui::Mode::Unscaled(scale) => 480.0 / scale,
                ui::Mode::Scaled => renderer.height as f64,
            });
        }
        None
    }

    // Events
    fn on_scroll(&mut self, _x: f64, _y: f64) {

    }

    fn is_closable(&self) -> bool {
        true
    }
}

pub struct AudioSettingsMenu {
    vars: Rc<console::Vars>,
    elements: Option<UIElements>
}

impl AudioSettingsMenu {
    pub fn new(vars: Rc<console::Vars>) -> AudioSettingsMenu {
        AudioSettingsMenu {
            vars: vars,
            elements: None
        }
    }
}

impl super::Screen for AudioSettingsMenu {
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let mut elements = ui::Collection::new();

        let mut background = ui::Image::new(
            render::Renderer::get_texture(renderer.get_textures_ref(), "steven:solid"),
            0.0, 0.0, 854.0, 480.0,
            0.0, 0.0, 1.0, 1.0,
            0, 0, 0
        );
        background.set_a(100);
        let background = elements.add(ui_container.add(background));

        let master_volume = *self.vars.get(settings::CL_MASTER_VOLUME);

        let (btn_master_volume, txt_master_volume) = new_centered_button(&master_volume.to_string(), renderer, ui_container, -150.0, ui::VAttach::Middle);
        elements.add(btn_master_volume);
        elements.add(txt_master_volume);

        let (btn_done, txt_done) = new_centered_button("Done", renderer, ui_container, 50.0, ui::VAttach::Bottom);
        super::button_action(ui_container, btn_done.clone(), Some(txt_done.clone()), move | game, _ | {
            game.screen_sys.pop_screen();
        });
        elements.add(btn_done);
        elements.add(txt_done);

        self.elements = Some(UIElements {
            elements: elements,
            background: background,
        });

    }
    fn on_deactive(&mut self, _renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        {
            let elements = self.elements.as_mut().unwrap();
            elements.elements.remove_all(ui_container);
        }
        self.elements = None;
    }

    // Called every frame the screen is active
    fn tick(&mut self, _delta: f64, renderer: &mut render::Renderer, ui_container: &mut ui::Container) -> Option<Box<super::Screen>> {
        let elements = self.elements.as_mut().unwrap();
        {
            let mode = ui_container.mode;
            let background = ui_container.get_mut(&elements.background);
            background.set_width(match mode {
                ui::Mode::Unscaled(scale) => 854.0 / scale,
                ui::Mode::Scaled => renderer.width as f64,
            });
            background.set_height(match mode {
                ui::Mode::Unscaled(scale) => 480.0 / scale,
                ui::Mode::Scaled => renderer.height as f64,
            });
        }
        None
    }

    // Events
    fn on_scroll(&mut self, _x: f64, _y: f64) {

    }

    fn is_closable(&self) -> bool {
        true
    }
}
