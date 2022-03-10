use anyhow::{anyhow, Result};
use tauri_hotkey::{Hotkey, HotkeyManager, Key, Modifier};

pub struct Helper {
    hkm: HotkeyManager,
}

impl Helper {
    pub fn new() -> Helper {
        Helper {
            hkm: HotkeyManager::new(),
        }
    }

    pub fn register_hk<F>(
        self: &mut Self,
        modifiers: Vec<Modifier>,
        keys: Vec<Key>,
        cb: F,
    ) -> Result<()>
    where
        F: 'static + FnMut() + Send,
    {
        self.hkm
            .register(Hotkey { modifiers, keys }, cb)
            .map_err(|e| anyhow!("{:?}", e))
    }
}
