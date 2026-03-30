use anyhow::Context;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub fn build_main_window(event_loop: &EventLoop<()>) -> anyhow::Result<Window> {
    WindowBuilder::new()
        .with_title("Spatial File Browser")
        .with_inner_size(PhysicalSize::new(1280, 800))
        .build(event_loop)
        .context("failed to create main window")
}
