use std::time::Instant;

use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::{window::Window, event::*};

use crate::render::Renderer;

mod debug_window;

pub struct MineConeUi {
    start_time: Instant,
    platform: Platform,
    render_pass: RenderPass,

    debug_panel_open: bool,
}

impl MineConeUi {
    pub fn new(window: &Window, renderer: &Renderer) -> Self {
        let size = window.inner_size();

        MineConeUi {
            start_time: Instant::now(),
            platform: Platform::new(PlatformDescriptor {
                physical_width: size.width,
                physical_height: size.height,
                scale_factor: window.scale_factor(),
                font_definitions: FontDefinitions::default(),
                style: Default::default(),
            }),
            render_pass: RenderPass::new(renderer.device(), renderer.surface_format(), 1),
            debug_panel_open: false,
        }
    }

    fn windows(&self) {
        if self.debug_panel_open {
            debug_window::debug_window(&self.platform.context());
        }
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        self.platform.handle_event(event);
    }

    pub fn input(&mut self, event: &WindowEvent) {
        if let WindowEvent::KeyboardInput {
            input: KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::F3),
                ..
            },
            ..
        } = event {
            self.debug_panel_open = !self.debug_panel_open;
        }
    }

    pub fn frame_update(&mut self, window: &Window, renderer: &Renderer) {
        self.platform.update_time(self.start_time.elapsed().as_secs_f64());

        let size = window.inner_size();
        let screen_descriptor = ScreenDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor() as f32,
        };

        self.platform.begin_frame();

        self.windows();

        let output = self.platform.end_frame(Some(window));
        let paint_jobs = self.platform.context().tessellate(output.shapes);
        let tdelta = output.textures_delta;

        let device = renderer.device();
        let queue = renderer.queue();

        let (_, view) = renderer.output_surface().expect("render pass has not been started");

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder"),
        });

        self.render_pass.add_textures(device, queue, &tdelta).unwrap();
        self.render_pass.update_buffers(device, queue, &paint_jobs, &screen_descriptor);
        self.render_pass.execute(
            &mut encoder,
            &view,
            &paint_jobs,
            &screen_descriptor,
            None,
        ).unwrap();

        queue.submit(std::iter::once(encoder.finish()));

        self.render_pass.remove_textures(tdelta).unwrap();
    }
}