use console;
use render;
use ui;
use settings;

use std::sync::{Arc, Mutex};

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

pub struct UIElement {
    elements: ui::Collection
    // TODO: Add background of some sort
}

pub struct SettingsMenu {
    console: Arc<Mutex<console::Console>>,
    elements: Option<UIElement>,
    show_disconnect_button: bool
}

impl SettingsMenu {
    pub fn new(console: Arc<Mutex<console::Console>>, show_disconnect_button: bool) -> SettingsMenu {
        SettingsMenu {
            console: console,
            elements: None,
            show_disconnect_button: show_disconnect_button
        }
    }
}

impl super::Screen for SettingsMenu {
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let mut elements = ui::Collection::new();

        // From top and down
        let (btn_audio_settings, txt_audio_settings) = new_submenu_button("Audio settings...", renderer, ui_container, -160.0, -50.0);
        super::button_action(ui_container, btn_audio_settings.clone(), Some(txt_audio_settings.clone()), move | game, _ | {
            game.screen_sys.add_screen(Box::new(AudioSettingsMenu::new(game.console.clone())));
        });
        elements.add(btn_audio_settings);
        elements.add(txt_audio_settings);

        let (btn_video_settings, txt_video_settings) = new_submenu_button("Video settings...", renderer, ui_container, 160.0, -50.0);
        super::button_action(ui_container, btn_video_settings.clone(), Some(txt_video_settings.clone()), move | game, _ | {
            game.screen_sys.add_screen(Box::new(VideoSettingsMenu::new(game.console.clone())));
        });
        elements.add(btn_video_settings);
        elements.add(txt_video_settings);

        let (btn_controls_settings, txt_controls_settings) = new_submenu_button("Controls...", renderer, ui_container, 160.0, 0.0);
        super::button_action(ui_container, btn_controls_settings.clone(), Some(txt_controls_settings.clone()), move | game, _ | {
            // TODO: Implement this...
        });
        elements.add(btn_controls_settings);
        elements.add(txt_controls_settings);

        let (btn_locale_settings, txt_locale_settings) = new_submenu_button("Language...", renderer, ui_container, -160.0, 0.0);
        super::button_action(ui_container, btn_locale_settings.clone(), Some(txt_locale_settings.clone()), move | game, _ | {
            // TODO: Implement this...
        });
        elements.add(btn_locale_settings);
        elements.add(txt_locale_settings);

        // Center bottom items
        let (mut btn_back_to_game, mut txt_back_to_game) = new_centered_button("Done", renderer, ui_container, 50.0, ui::VAttach::Bottom);
        super::button_action(ui_container, btn_back_to_game.clone(), Some(txt_back_to_game.clone()), move | game, _ | {
            game.screen_sys.pop_screen();
            game.focused = true;
        });
        elements.add(btn_back_to_game);
        elements.add(txt_back_to_game);

        if self.show_disconnect_button {
            let (mut btn_exit_game, mut txt_exit_game) = new_centered_button("Disconnect", renderer, ui_container, 100.0, ui::VAttach::Bottom);
            super::button_action(ui_container, btn_exit_game.clone(), Some(txt_exit_game.clone()), move | game, _ | {
                game.server.disconnect(None);
                game.screen_sys.replace_screen(Box::new(super::ServerList::new(None)));
            });
            elements.add(btn_exit_game);
            elements.add(txt_exit_game);
        }

        self.elements = Some(UIElement {
            elements: elements
        });

    }
    fn on_deactive(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        {
            let elements = self.elements.as_mut().unwrap();
            elements.elements.remove_all(ui_container);
        }
        self.elements = None;
    }

    // Called every frame the screen is active
    fn tick(&mut self, delta: f64, renderer: &mut render::Renderer, ui_container: &mut ui::Container) -> Option<Box<super::Screen>> {
        None
    }

    // Events
    fn on_scroll(&mut self, x: f64, y: f64) {

    }

    fn is_closable(&self) -> bool {
        true
    }
}

pub struct VideoSettingsMenu {
    console: Arc<Mutex<console::Console>>,
    elements: Option<UIElement>,
    fps_bounds: Vec<i64>
}

impl VideoSettingsMenu {
    pub fn new(console: Arc<Mutex<console::Console>>) -> VideoSettingsMenu {
        VideoSettingsMenu {
            console: console,
            elements: None,
            fps_bounds: vec!(60, 120, 144, -1)
        }
    }
}

impl super::Screen for VideoSettingsMenu {
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let mut elements = ui::Collection::new();

        // Load defaults
        let (r_max_fps, r_fov, r_vsync) = {
            let console = self.console.lock().unwrap();
            (
                console.get(settings::R_MAX_FPS).clone(),
                console.get(settings::R_FOV).clone(),
                console.get(settings::R_VSYNC).clone()
            )
        };

        // Setting buttons
        // TODO: Slider
        let (btn_fov, txt_fov) = new_submenu_button(get_matched_str!("FOV: {}", r_fov, 90 => "Normal", 110 => "Quake pro"), renderer, ui_container, -160.0, -50.0);
        elements.add(btn_fov);
        elements.add(txt_fov);

        let (btn_vsync, txt_vsync) = new_submenu_button(get_bool_str!("VSync: {}", r_vsync, "Enabled", "Disabled"), renderer, ui_container, -160.0, 0.0);
        elements.add(txt_vsync.clone());
        super::button_action(ui_container, btn_vsync.clone(), Some(txt_vsync.clone()), move | game, ui_container | {
            let mut console = game.console.lock().unwrap();
            let r_vsync = !console.get(settings::R_VSYNC);
            let txt_vsync = ui_container.get_mut(&txt_vsync);
            txt_vsync.set_text(&game.renderer, get_bool_str!("VSync: {}", r_vsync, "Enabled", "Disabled"));
            console.set(settings::R_VSYNC, r_vsync);
        });
        elements.add(btn_vsync);

        // TODO: Slider
        let (btn_fps_cap, txt_fps_cap) = new_submenu_button(get_matched_str!("FPS cap: {}", r_max_fps, 0 => "Unlimited", 15 => "Potato"), renderer, ui_container, 160.0, 0.0);
        elements.add(btn_fps_cap);
        elements.add(txt_fps_cap);

        let (mut btn_done, mut txt_done) = new_centered_button("Done", renderer, ui_container, 50.0, ui::VAttach::Bottom);
        super::button_action(ui_container, btn_done.clone(), Some(txt_done.clone()), move | game, _ | {
            game.screen_sys.pop_screen();
        });
        elements.add(btn_done);
        elements.add(txt_done);
        self.elements = Some(UIElement {
            elements: elements
        });

    }
    fn on_deactive(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        {
            let elements = self.elements.as_mut().unwrap();
            elements.elements.remove_all(ui_container);
        }
        self.elements = None;
    }

    // Called every frame the screen is active
    fn tick(&mut self, delta: f64, renderer: &mut render::Renderer, ui_container: &mut ui::Container) -> Option<Box<super::Screen>> {
        None
    }

    // Events
    fn on_scroll(&mut self, x: f64, y: f64) {

    }

    fn is_closable(&self) -> bool {
        true
    }
}

pub struct AudioSettingsMenu {
    console: Arc<Mutex<console::Console>>,
    elements: Option<UIElement>
}

impl AudioSettingsMenu {
    pub fn new(console: Arc<Mutex<console::Console>>) -> AudioSettingsMenu {
        AudioSettingsMenu {
            console: console,
            elements: None
        }
    }
}

impl super::Screen for AudioSettingsMenu {
    fn on_active(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        let mut elements = ui::Collection::new();

        let master_volume = {
            let console = self.console.lock().unwrap();
            (console.get(settings::CL_MASTER_VOLUME).clone())
        };

        let (mut btn_master_volume, mut txt_master_volume) = new_centered_button(master_volume.to_string().as_ref(), renderer, ui_container, -150.0, ui::VAttach::Middle);
        elements.add(btn_master_volume);
        elements.add(txt_master_volume);

        let (mut btn_done, mut txt_done) = new_centered_button("Done", renderer, ui_container, 50.0, ui::VAttach::Bottom);
        super::button_action(ui_container, btn_done.clone(), Some(txt_done.clone()), move | game, _ | {
            game.screen_sys.pop_screen();
        });
        elements.add(btn_done);
        elements.add(txt_done);

        self.elements = Some(UIElement {
            elements: elements
        });

    }
    fn on_deactive(&mut self, renderer: &mut render::Renderer, ui_container: &mut ui::Container) {
        {
            let elements = self.elements.as_mut().unwrap();
            elements.elements.remove_all(ui_container);
        }
        self.elements = None;
    }

    // Called every frame the screen is active
    fn tick(&mut self, delta: f64, renderer: &mut render::Renderer, ui_container: &mut ui::Container) -> Option<Box<super::Screen>> {
        None
    }

    // Events
    fn on_scroll(&mut self, x: f64, y: f64) {

    }

    fn is_closable(&self) -> bool {
        true
    }
}
