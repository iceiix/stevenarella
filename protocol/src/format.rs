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

use std::fmt;
use std::mem;

#[derive(Debug, Clone)]
pub enum Component {
    Text(TextComponent),
}

impl Component {
    pub fn from_string(str: &str) -> Self {
        let mut component;
        match serde_json::from_str::<serde_json::Value>(str) {
            Ok(value) => component = Component::from_value(&value),
            // Sometimes mojang sends a literal string, so we should interpret it literally
            Err(_) => {
                component = Component::Text(TextComponent::new(str));
                convert_legacy(&mut component);
            }
        }
        component
    }

    pub fn from_value(v: &serde_json::Value) -> Self {
        let mut modifier = Modifier::from_value(v);
        if let Some(val) = v.as_str() {
            Component::Text(TextComponent {
                text: val.to_owned(),
                modifier,
            })
        } else if v.get("text").is_some() {
            Component::Text(TextComponent::from_value(v, modifier))
        } else if v.get("translate").is_some() {
            // TODO: translations
            Component::Text(TextComponent::new(
                v.get("translate").unwrap().as_str().unwrap(),
            ))
        } else {
            modifier.color = Some(Color::RGB(255, 0, 0));
            Component::Text(TextComponent {
                text: "UNHANDLED".to_owned(),
                modifier,
            })
        }
    }

    pub fn to_value(&self) -> serde_json::Value {
        unimplemented!()
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Component::Text(ref txt) => write!(f, "{}", txt),
        }
    }
}

impl Default for Component {
    fn default() -> Self {
        Component::Text(TextComponent {
            text: "".to_owned(),
            modifier: Default::default(),
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct Modifier {
    pub extra: Option<Vec<Component>>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
    pub color: Option<Color>,
}

// TODO: Missing events click/hover/insert

impl Modifier {
    pub fn from_value(v: &serde_json::Value) -> Self {
        let mut m = Modifier {
            bold: v.get("bold").and_then(|v| v.as_bool()),
            italic: v.get("italic").and_then(|v| v.as_bool()),
            underlined: v.get("underlined").and_then(|v| v.as_bool()),
            strikethrough: v.get("strikethrough").and_then(|v| v.as_bool()),
            obfuscated: v.get("obfuscated").and_then(|v| v.as_bool()),
            color: v
                .get("color")
                .and_then(|v| v.as_str())
                .map(|v| Color::from_string(&v.to_owned())),
            extra: Option::None,
        };
        if let Some(extra) = v.get("extra") {
            if let Some(data) = extra.as_array() {
                let mut ex = Vec::new();
                for e in data {
                    ex.push(Component::from_value(e));
                }
                m.extra = Some(ex);
            }
        }
        m
    }

    pub fn to_value(&self) -> serde_json::Value {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct TextComponent {
    pub text: String,
    pub modifier: Modifier,
}

impl TextComponent {
    pub fn new(val: &str) -> TextComponent {
        TextComponent {
            text: val.to_owned(),
            modifier: Modifier {
                ..Default::default()
            },
        }
    }

    pub fn from_value(v: &serde_json::Value, modifier: Modifier) -> Self {
        TextComponent {
            text: v.get("text").unwrap().as_str().unwrap_or("").to_owned(),
            modifier,
        }
    }

    pub fn to_value(&self) -> serde_json::Value {
        unimplemented!()
    }
}

impl fmt::Display for TextComponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)?;
        if let Some(ref extra) = self.modifier.extra {
            for c in extra {
                write!(f, "{}", c)?;
            }
        }
        Result::Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    RGB(u8, u8, u8),
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Color::Black => "black".to_owned(),
                Color::DarkBlue => "dark_blue".to_owned(),
                Color::DarkGreen => "dark_green".to_owned(),
                Color::DarkAqua => "dark_aqua".to_owned(),
                Color::DarkRed => "dark_red".to_owned(),
                Color::DarkPurple => "dark_purple".to_owned(),
                Color::Gold => "gold".to_owned(),
                Color::Gray => "gray".to_owned(),
                Color::DarkGray => "dark_gray".to_owned(),
                Color::Blue => "blue".to_owned(),
                Color::Green => "green".to_owned(),
                Color::Aqua => "aqua".to_owned(),
                Color::Red => "red".to_owned(),
                Color::LightPurple => "light_purple".to_owned(),
                Color::Yellow => "yellow".to_owned(),
                Color::White => "white".to_owned(),
                Color::RGB(r, g, b) => format!("#{:02X}{:02X}{:02X}", r, g, b),
            }
        )
    }
}

impl Color {
    fn from_string(val: &str) -> Self {
        match val {
            "black" => Color::Black,
            "dark_blue" => Color::DarkBlue,
            "dark_green" => Color::DarkGreen,
            "dark_aqua" => Color::DarkAqua,
            "dark_red" => Color::DarkRed,
            "dark_purple" => Color::DarkPurple,
            "gold" => Color::Gold,
            "gray" => Color::Gray,
            "dark_gray" => Color::DarkGray,
            "blue" => Color::Blue,
            "green" => Color::Green,
            "aqua" => Color::Aqua,
            "red" => Color::Red,
            "light_purple" => Color::LightPurple,
            "yellow" => Color::Yellow,
            val if val.len() == 7 && val.as_bytes()[0] == b'#' => {
                let r = match u8::from_str_radix(&val[1..3], 16) {
                    Ok(r) => r,
                    Err(_) => return Color::White,
                };
                let g = match u8::from_str_radix(&val[3..5], 16) {
                    Ok(g) => g,
                    Err(_) => return Color::White,
                };
                let b = match u8::from_str_radix(&val[5..7], 16) {
                    Ok(b) => b,
                    Err(_) => return Color::White,
                };
                Color::RGB(r, g, b)
            }
            "white" => Color::White,
            _ => Color::White,
        }
    }

    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match *self {
            Color::Black => (0, 0, 0),
            Color::DarkBlue => (0, 0, 170),
            Color::DarkGreen => (0, 170, 0),
            Color::DarkAqua => (0, 170, 170),
            Color::DarkRed => (170, 0, 0),
            Color::DarkPurple => (170, 0, 170),
            Color::Gold => (255, 170, 0),
            Color::Gray => (170, 170, 170),
            Color::DarkGray => (85, 85, 85),
            Color::Blue => (85, 85, 255),
            Color::Green => (85, 255, 85),
            Color::Aqua => (85, 255, 255),
            Color::Red => (255, 85, 85),
            Color::LightPurple => (255, 85, 255),
            Color::Yellow => (255, 255, 85),
            Color::White => (255, 255, 255),
            Color::RGB(r, g, b) => (r, g, b),
        }
    }
}

#[test]
fn test_color_from() {
    let test = Color::from_string(&"#FF0000".to_owned());
    match test {
        Color::RGB(r, g, b) => assert!(r == 255 && g == 0 && b == 0),
        _ => panic!("Wrong type"),
    }
    let test = Color::from_string(&"#123456".to_owned());
    match test {
        Color::RGB(r, g, b) => assert!(r == 0x12 && g == 0x34 && b == 0x56),
        _ => panic!("Wrong type"),
    }
    let test = Color::from_string(&"red".to_owned());
    match test {
        Color::Red => {}
        _ => panic!("Wrong type"),
    }
    let test = Color::from_string(&"dark_blue".to_owned());
    match test {
        Color::DarkBlue => {}
        _ => panic!("Wrong type"),
    }
}

const LEGACY_CHAR: char = '§';

pub fn convert_legacy(c: &mut Component) {
    match *c {
        Component::Text(ref mut txt) => {
            if let Some(ref mut extra) = txt.modifier.extra.as_mut() {
                for e in extra.iter_mut() {
                    convert_legacy(e);
                }
            }
            if txt.text.contains(LEGACY_CHAR) {
                let mut parts = Vec::new();
                let mut last = 0;
                let mut current = TextComponent::new("");
                {
                    let mut iter = txt.text.char_indices();
                    while let Some((i, c)) = iter.next() {
                        if c == LEGACY_CHAR {
                            let next = match iter.next() {
                                Some(val) => val,
                                None => break,
                            };
                            let color_char = next.1.to_lowercase().next().unwrap();
                            current.text = txt.text[last..i].to_owned();
                            last = next.0 + 1;

                            let mut modifier = if (color_char >= 'a' && color_char <= 'f')
                                || (color_char >= '0' && color_char <= '9')
                            {
                                Default::default()
                            } else {
                                current.modifier.clone()
                            };

                            let new = TextComponent::new("");
                            parts.push(Component::Text(mem::replace(&mut current, new)));

                            match color_char {
                                '0' => modifier.color = Some(Color::Black),
                                '1' => modifier.color = Some(Color::DarkBlue),
                                '2' => modifier.color = Some(Color::DarkGreen),
                                '3' => modifier.color = Some(Color::DarkAqua),
                                '4' => modifier.color = Some(Color::DarkRed),
                                '5' => modifier.color = Some(Color::DarkPurple),
                                '6' => modifier.color = Some(Color::Gold),
                                '7' => modifier.color = Some(Color::Gray),
                                '8' => modifier.color = Some(Color::DarkGray),
                                '9' => modifier.color = Some(Color::Blue),
                                'a' => modifier.color = Some(Color::Green),
                                'b' => modifier.color = Some(Color::Aqua),
                                'c' => modifier.color = Some(Color::Red),
                                'd' => modifier.color = Some(Color::LightPurple),
                                'e' => modifier.color = Some(Color::Yellow),
                                'f' => modifier.color = Some(Color::White),
                                'k' => modifier.obfuscated = Some(true),
                                'l' => modifier.bold = Some(true),
                                'm' => modifier.strikethrough = Some(true),
                                'n' => modifier.underlined = Some(true),
                                'o' => modifier.italic = Some(true),
                                'r' => {}
                                _ => println!(
                                    "warning: unsupported color code {:?} in text '{}'",
                                    color_char, txt
                                ),
                            }

                            current.modifier = modifier;
                        }
                    }
                }
                if last < txt.text.len() {
                    current.text = txt.text[last..].to_owned();
                    parts.push(Component::Text(current));
                }

                let old = mem::replace(&mut txt.modifier.extra, Some(parts));
                if let Some(old_extra) = old {
                    if let Some(ref mut extra) = txt.modifier.extra.as_mut() {
                        extra.extend(old_extra);
                    }
                }
                txt.text = "".to_owned();
            }
        }
    }
}
