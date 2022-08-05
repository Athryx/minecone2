#![feature(once_cell)]
#![feature(drain_filter)]
#![feature(test)]

#[macro_use]
extern crate log;

use winit::{
	event_loop::EventLoop,
	window::WindowBuilder,
	dpi::PhysicalSize,
};

mod game;
mod render;
mod assets;
mod prelude;

fn main() {
    pretty_env_logger::init();

    let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title("Minecone")
		.with_inner_size(PhysicalSize::new(1280, 720))
		.build(&event_loop)
		.unwrap();

    let mut game = game::Game::new(60, &window);

    event_loop.run(move |event, _, control_flow| {
		*control_flow = game.event_update(event);
	});
}
