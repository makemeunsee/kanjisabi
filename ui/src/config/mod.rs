use std::{path::PathBuf, sync::mpsc::Receiver, time::Duration};

use anyhow::{anyhow, Result};
use config::File;
use device_query::Keycode;
use directories::BaseDirs;
use log::warn;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{de::Error, Deserialize, Deserializer};
use serde_with::{serde_as, DeserializeAs};

const CONFIG_FILE: &str = "kanjisabi.toml";

pub fn load_config() -> Result<KSConfig> {
    config::Config::builder()
        .add_source(File::from(config_path()))
        .build()
        .map_err(|e| {
            warn!("Failed to load config file: {:?}", e);
            e
        })?
        .try_deserialize()
        .map_err(|e| {
            warn!("Incompatible configuration: {:?}", e);
            anyhow!("Incompatible configuration: {:?}", e)
        })
}

pub fn watch_config() -> Result<(Receiver<()>, RecommendedWatcher)> {
    let (config_tx, config_rx) = std::sync::mpsc::channel();
    let mut config_watcher = RecommendedWatcher::new(
        move |result: notify::Result<Event>| match result {
            Ok(event) if event.paths.contains(&config_path()) && event.kind.is_modify() => {
                let _ = config_tx.send(());
            }
            _ => {}
        },
        notify::Config::default().with_poll_interval(Duration::from_secs(2)),
    )?;
    config_watcher.watch(&config_dir_path(), RecursiveMode::Recursive)?;
    Ok((config_rx, config_watcher))
}

fn config_dir_path() -> PathBuf {
    BaseDirs::new().unwrap().config_dir().to_path_buf()
}

fn config_path() -> PathBuf {
    let mut path = config_dir_path();
    path.push(CONFIG_FILE);
    path
}

#[derive(Deserialize, Debug, Default)]
pub struct KSConfig {
    #[serde(default = "LinderaConfig::default")]
    pub lindera: LinderaConfig,
    #[serde(default = "Font::default")]
    pub font: Font,
    #[serde(default = "Colors::default")]
    pub colors: Colors,
    #[serde(default = "Preproc::default")]
    pub preproc: Preproc,
    #[serde(default = "Keys::default")]
    pub keys: Keys,
}

// Lindera server integration

fn default_lindera_address() -> String {
    "0.0.0.0:3333".to_owned()
}

#[derive(Deserialize, Debug)]
pub struct LinderaConfig {
    #[serde(default = "default_lindera_address")]
    pub server_address: String,
}

impl Default for LinderaConfig {
    fn default() -> Self {
        Self {
            server_address: default_lindera_address(),
        }
    }
}

// font

fn default_family() -> Option<String> {
    None
}

fn default_style() -> Option<String> {
    None
}

#[derive(Deserialize, Debug, Default)]
pub struct Font {
    #[serde(default = "default_family")]
    pub family: Option<String>,
    #[serde(default = "default_style")]
    pub style: Option<String>,
}

// colors

fn default_capture() -> u32 {
    0x20002000
}

fn default_highlight() -> u32 {
    0x20200000
}

fn default_hint() -> u32 {
    0xFF32FF00
}

fn default_hint_bg() -> u32 {
    0xC0000024
}

// TODO possible to tell serde to re-use defaults?
#[derive(Deserialize, Debug)]
pub struct Colors {
    // #[serde(default = "default_capture")]
    pub capture: u32,
    // #[serde(default = "default_highlight")]
    pub highlight: u32,
    // #[serde(default = "default_hint")]
    pub hint: u32,
    // #[serde(default = "default_hint_bg")]
    pub hint_bg: u32,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            capture: default_capture(),
            highlight: default_highlight(),
            hint: default_hint(),
            hint_bg: default_hint_bg(),
        }
    }
}

// preproc

fn default_contrast() -> f32 {
    100.
}

#[derive(Deserialize, Debug)]
pub struct Preproc {
    #[serde(default = "default_contrast")]
    pub contrast: f32,
}

impl Default for Preproc {
    fn default() -> Self {
        Self {
            contrast: default_contrast(),
        }
    }
}

struct LocalKeycode;

impl<'de> DeserializeAs<'de, Keycode> for LocalKeycode {
    fn deserialize_as<D>(deserializer: D) -> Result<Keycode, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Keycode>()
            .map_err(|e| D::Error::custom(format!("{} on \"{}\"", e, s)))
    }
}
// keys

fn default_trigger() -> Vec<Keycode> {
    vec![Keycode::LControl, Keycode::LAlt]
}

fn default_quit() -> Vec<Keycode> {
    vec![Keycode::LControl, Keycode::LAlt, Keycode::Escape]
}

fn default_font_up() -> Vec<Keycode> {
    vec![Keycode::LShift]
}

fn default_font_down() -> Vec<Keycode> {
    vec![Keycode::RShift]
}

fn default_next_hint() -> Vec<Keycode> {
    vec![Keycode::LControl]
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Keys {
    #[serde_as(as = "Vec<LocalKeycode>")]
    #[serde(default = "default_trigger")]
    pub trigger: Vec<Keycode>,
    #[serde_as(as = "Vec<LocalKeycode>")]
    #[serde(default = "default_quit")]
    pub quit: Vec<Keycode>,
    #[serde_as(as = "Vec<LocalKeycode>")]
    #[serde(default = "default_font_up")]
    pub font_up: Vec<Keycode>,
    #[serde_as(as = "Vec<LocalKeycode>")]
    #[serde(default = "default_font_down")]
    pub font_down: Vec<Keycode>,
    #[serde_as(as = "Vec<LocalKeycode>")]
    #[serde(default = "default_next_hint")]
    pub next_hint: Vec<Keycode>,
}

impl Default for Keys {
    fn default() -> Self {
        Self {
            trigger: default_trigger(),
            quit: default_quit(),
            font_up: default_font_up(),
            font_down: default_font_up(),
            next_hint: default_next_hint(),
        }
    }
}
