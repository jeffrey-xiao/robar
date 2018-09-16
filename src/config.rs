use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::Read;
use std::path::Path;
use toml;
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};
use std::fmt;

// TODO: Add icon, follow, fill direction
#[derive(Deserialize, Debug)]
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
    pub height_relative: u32,
    pub height_absolute: u32,
    #[serde(default)]
    pub width_relative: u32,
    pub width_absolute: u32,

    #[serde(default = "GlobalConfig::default_timeout")]
    pub timeout: u64,
}

impl GlobalConfig {
    fn default_timeout() -> u64 { 1000 }

    pub fn width_to_margin(&self) -> u32 {
        self.width_to_border() + 2 * self.margin
    }

    pub fn width_to_border(&self) -> u32 {
        self.width_to_padding() + 2 * self.border
    }

    pub fn width_to_padding(&self) -> u32 {
        self.width() + 2 * self.padding
    }

    pub fn height_to_margin(&self) -> u32 {
        self.height_to_border() + 2 * self.margin
    }

    pub fn height_to_border(&self) -> u32 {
        self.height_to_padding() + 2 * self.border
    }

    pub fn height_to_padding(&self) -> u32 {
        self.height() + 2 * self.padding
    }

    pub fn width(&self) -> u32 {
        self.width_absolute
    }

    pub fn height(&self) -> u32 {
        self.height_absolute
    }

    pub fn x(&self) -> u32 {
        self.x_center() - self.width_to_margin() / 2
    }

    pub fn y(&self) -> u32 {
        self.y_center() - self.height_to_margin() / 2
    }

    pub fn x_center(&self) -> u32 {
        self.x_center_absolute
    }

    pub fn y_center(&self) -> u32 {
        self.y_center_absolute
    }
}

pub struct ColorConfig {
    pub foreground: u32,
    pub background: u32,
    pub border: u32,
}

impl ColorConfig {
    pub fn new(foreground: String, background: String, border: String) -> Self {
        let foreground = u32::from_str_radix(&foreground[1..], 16).expect("Could not parse color");
        let background = u32::from_str_radix(&background[1..], 16).expect("Could not parse color");
        let border = u32::from_str_radix(&border[1..], 16).expect("Could not parse color");

        ColorConfig { foreground, background, border }
    }
}

impl<'de> Deserialize<'de> for ColorConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Foreground, Background, Border }

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
                let foreground = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let background = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let border = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(ColorConfig::new(foreground, background, border))
            }

            fn visit_map<V>(self, mut map: V) -> Result<ColorConfig, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut foreground = None;
                let mut background = None;
                let mut border = None;
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

                let foreground = foreground.ok_or_else(|| de::Error::missing_field("foreground"))?;
                let background = background.ok_or_else(|| de::Error::missing_field("background"))?;
                let border = border.ok_or_else(|| de::Error::missing_field("border"))?;
                Ok(ColorConfig::new(foreground, background, border))
            }
        }

        const FIELDS: &'static [&'static str] = &["secs", "nanos"];
        deserializer.deserialize_struct("ColorConfig", FIELDS, ColorConfigVisitor)
    }
}

pub fn parse_config<P>(config_path: P) -> (GlobalConfig, HashMap<String, ColorConfig>)
where
    P: AsRef<Path>,
{
    let mut config_file = File::open(&config_path).expect("File does not exist.");
    let mut raw_config = String::new();
    config_file.read_to_string(&mut raw_config).expect("Failed to read config.");

    let toml_value = raw_config.parse::<toml::Value>().expect("Failed to parse config file.");
    let mut toml_table = match toml_value {
        toml::Value::Table(table) => table,
        _ => panic!("Failed to parse toml config file: Expected table at root."),
    };

    let global_value = toml_table.remove("global").take().expect("Missing global section.");
    let global_config = global_value.try_into::<GlobalConfig>().expect("Failed to parse.");

    let color_values = toml_table.remove("colors").take().expect("Missing colors section.");
    let mut color_configs = HashMap::new();

    let color_values = match color_values {
        toml::Value::Table(table) => table,
        _ => panic!("Failed to parse toml config file: Expected array in global table."),
    };

    for (profile_name, color_value) in color_values {
        color_configs.insert(profile_name, color_value.try_into::<ColorConfig>().expect("Failed to parse"));
    }

    (global_config, color_configs)
}
