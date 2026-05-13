use web_sys::window;

pub fn get_token_from_storage() -> Option<String> {
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("ppt_token").ok().flatten())
}

pub fn save_token_to_storage(token: &str) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item("ppt_token", token);
    }
}

pub fn clear_token_from_storage() {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.remove_item("ppt_token");
    }
}
