use egui::{Window, Context};

pub fn debug_window(context: &Context) {
    Window::new("Debug Window").show(context, |ui| {
        ui.label("Hello, World!");
    });
}