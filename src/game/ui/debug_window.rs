use std::{sync::LazyLock, collections::BTreeMap};

use egui::{Window, Context};
use parking_lot::Mutex;

static debug_info: LazyLock<Mutex<BTreeMap<String, String>>> = LazyLock::new(|| Mutex::new(BTreeMap::new()));

pub fn debug_string(label: &str, data: String) {
    let mut map = debug_info.lock();

    map.insert(String::from(label), data);
}

pub fn debug_display<T: ToString>(label: &str, data: &T) {
    debug_string(label, data.to_string());
}

pub fn debug_window(context: &Context) {
    Window::new("Debug Window").show(context, |ui| {
        let map = debug_info.lock();

        for (label, data) in map.iter() {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.label(data);
            });
        }
    });
}