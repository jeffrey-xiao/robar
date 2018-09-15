use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::Read;
use std::path::Path;
use toml;

// TODO: Add icon, transparency, follow, fill direction
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
}

impl GlobalConfig {
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

#[derive(Deserialize, Debug)]
pub struct ColorConfig {
    foreground: String,
    background: String,
    border: String,
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
