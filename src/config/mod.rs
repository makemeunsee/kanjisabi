use device_query::Keycode;
use serde::{de::Error, Deserialize, Deserializer};
use serde_with::{serde_as, DeserializeAs};

// #[derive(Deserialize, Debug)]
// pub struct Colors {
//     #[serde(default)]
//     capture: u32,
//     #[serde(default)]
//     highlight: u32,
//     #[serde(default)]
//     hint: u32,
//     #[serde(default)]
//     hint_bg: u32,
// }

struct LocalKeycode;

impl<'de> DeserializeAs<'de, Keycode> for LocalKeycode {
    fn deserialize_as<D>(deserializer: D) -> Result<Keycode, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)? {
            s => s
                .parse::<Keycode>()
                .map_err(|e| D::Error::custom(format!("{} on \"{}\"", e, s))),
        }
    }
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Keys {
    #[serde_as(as = "Vec<LocalKeycode>")]
    trigger: Vec<Keycode>,
    #[serde_as(as = "Vec<LocalKeycode>")]
    quit: Vec<Keycode>,
    #[serde_as(as = "Vec<LocalKeycode>")]
    font_up: Vec<Keycode>,
    #[serde_as(as = "Vec<LocalKeycode>")]
    font_down: Vec<Keycode>,
}
