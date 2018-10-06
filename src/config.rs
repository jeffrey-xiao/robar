use super::Error;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::Read;
use std::path::Path;
use toml;

#[derive(Copy, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// TODO: Add icon
#[derive(Copy, Clone, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub x_center_relative: f32,
    pub x_center_absolute: u32,
    #[serde(default)]
    pub y_center_relative: f32,
    pub y_center_absolute: u32,

    #[serde(default)]
    pub margin: u32,
    #[serde(default)]
    pub border: u32,
    #[serde(default)]
    pub padding: u32,

    #[serde(default)]
    pub height_relative: f32,
    pub height_absolute: u32,
    #[serde(default)]
    pub width_relative: f32,
    pub width_absolute: u32,

    #[serde(default = "GlobalConfig::default_timeout")]
    pub timeout: u64,

    pub fill_direction: Direction,
}

impl GlobalConfig {
    fn default_timeout() -> u64 {
        1000
    }

    pub fn total_width(&self, screen_width: u32) -> u32 {
        2 * (self.border + self.margin + self.padding) + self.width(screen_width)
    }

    pub fn total_height(&self, screen_height: u32) -> u32 {
        2 * (self.border + self.margin + self.padding) + self.height(screen_height)
    }

    pub fn width(&self, screen_width: u32) -> u32 {
        self.width_absolute + f32::round(screen_width as f32 * self.width_relative) as u32
    }

    pub fn height(&self, screen_height: u32) -> u32 {
        self.height_absolute + f32::round(screen_height as f32 * self.height_relative) as u32
    }

    pub fn x(&self, screen_width: u32) -> u32 {
        self.x_center(screen_width) - self.total_width(screen_width) / 2
    }

    pub fn y(&self, screen_height: u32) -> u32 {
        self.y_center(screen_height) - self.total_height(screen_height) / 2
    }

    pub fn x_center(&self, screen_width: u32) -> u32 {
        self.x_center_absolute + f32::round(screen_width as f32 * self.x_center_relative) as u32
    }

    pub fn y_center(&self, screen_height: u32) -> u32 {
        self.y_center_absolute + f32::round(screen_height as f32 * self.y_center_relative) as u32
    }
}

#[derive(Copy, Clone)]
pub struct ColorConfig {
    pub foreground: u32,
    pub background: u32,
    pub border: u32,
}

impl ColorConfig {
    pub fn new(foreground: u32, background: u32, border: u32) -> Self {
        ColorConfig {
            foreground,
            background,
            border,
        }
    }
}

impl<'de> Deserialize<'de> for ColorConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Foreground,
            Background,
            Border,
        }

        struct ColorConfigVisitor;

        impl<'de> Visitor<'de> for ColorConfigVisitor {
            type Value = ColorConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ColorConfig")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<ColorConfig, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let foreground: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let foreground = u32::from_str_radix(&foreground[1..], 16).map_err(|_| {
                    de::Error::invalid_value(de::Unexpected::Str(&foreground), &"a hex color")
                })?;
                let background: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let background = u32::from_str_radix(&background[1..], 16).map_err(|_| {
                    de::Error::invalid_value(de::Unexpected::Str(&background), &"a hex color")
                })?;
                let border: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let border = u32::from_str_radix(&border[1..], 16).map_err(|_| {
                    de::Error::invalid_value(de::Unexpected::Str(&border), &"a hex color")
                })?;
                Ok(ColorConfig::new(foreground, background, border))
            }

            fn visit_map<V>(self, mut map: V) -> Result<ColorConfig, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut foreground: Option<String> = None;
                let mut background: Option<String> = None;
                let mut border: Option<String> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Foreground => {
                            if foreground.is_some() {
                                return Err(de::Error::duplicate_field("foreground"));
                            }
                            foreground = Some(map.next_value()?);
                        },
                        Field::Background => {
                            if background.is_some() {
                                return Err(de::Error::duplicate_field("background"));
                            }
                            background = Some(map.next_value()?);
                        },
                        Field::Border => {
                            if border.is_some() {
                                return Err(de::Error::duplicate_field("border"));
                            }
                            border = Some(map.next_value()?);
                        },
                    }
                }

                let foreground =
                    foreground.ok_or_else(|| de::Error::missing_field("foreground"))?;
                let foreground = u32::from_str_radix(&foreground[1..], 16).map_err(|_| {
                    de::Error::invalid_value(de::Unexpected::Str(&foreground), &"a hex color")
                })?;
                let background =
                    background.ok_or_else(|| de::Error::missing_field("background"))?;
                let background = u32::from_str_radix(&background[1..], 16).map_err(|_| {
                    de::Error::invalid_value(de::Unexpected::Str(&background), &"a hex color")
                })?;
                let border = border.ok_or_else(|| de::Error::missing_field("border"))?;
                let border = u32::from_str_radix(&border[1..], 16).map_err(|_| {
                    de::Error::invalid_value(de::Unexpected::Str(&border), &"a hex color")
                })?;

                Ok(ColorConfig::new(foreground, background, border))
            }
        }

        const FIELDS: &[&str] = &["secs", "nanos"];
        deserializer.deserialize_struct("ColorConfig", FIELDS, ColorConfigVisitor)
    }
}

pub fn parse_config<P>(
    config_path: P,
) -> super::Result<(GlobalConfig, HashMap<String, ColorConfig>)>
where
    P: AsRef<Path>,
{
    let mut config_file =
        File::open(&config_path).map_err(|err| Error::new("reading config", &err))?;
    let mut raw_config = String::new();
    config_file
        .read_to_string(&mut raw_config)
        .map_err(|err| Error::new("reading config", &err))?;

    let toml_value = raw_config
        .parse::<toml::Value>()
        .map_err(|err| Error::new("parsing config", &err))?;
    let mut toml_table = match toml_value {
        toml::Value::Table(table) => Ok(table),
        _ => {
            Err(Error::from_description(
                "parsing config",
                "Expected table at root of config.",
            ))
        },
    }?;

    let global_value = toml_table.remove("global").take().ok_or_else(|| {
        Error::from_description("parsing config", "Expected `global` section in config.")
    })?;
    let global_config = global_value
        .try_into::<GlobalConfig>()
        .map_err(|err| Error::new("parsing config", &err))?;

    let color_values = toml_table.remove("colors").take().ok_or_else(|| {
        Error::from_description("parsing config", "Expected `colors` section in config.")
    })?;

    let mut color_configs = HashMap::new();
    let color_values = match color_values {
        toml::Value::Table(table) => Ok(table),
        _ => {
            Err(Error::from_description(
                "parsing config",
                "Expected array in `colors` section.",
            ))
        },
    }?;

    for (profile_name, color_value) in color_values {
        let color_config = color_value.try_into::<ColorConfig>().map_err(|err| {
            Error::new(format!("parsing color profile `{}`", &profile_name), &err)
        })?;
        color_configs.insert(profile_name, color_config);
    }

    Ok((global_config, color_configs))
}
