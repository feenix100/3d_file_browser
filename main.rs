mod app;
mod domain;
mod fs;
mod platform;
mod render;
mod scene;
mod ui;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use app::commands::AppCommand;
use app::App;
use platform::input::InputMapper;
use render::renderer::Renderer;
use tracing::{error, info};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if let Err(err) = run() {
        error!("{err:#}");
    }
}

fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window = Arc::new(platform::window::build_main_window(&event_loop)?);

    let size = window.inner_size();
    let mut renderer = pollster::block_on(Renderer::new(window.clone(), size))?;
    let start_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut app = App::new(start_dir);
    let mut input = InputMapper::default();
    let mut last_redraw = Instant::now();

    app.dispatch(AppCommand::LoadDirectory(app.state.navigation.current_path.clone()));
    window.request_redraw();

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::CloseRequested = event {
                    elwt.exit();
                    return;
                }

                if let WindowEvent::Resized(new_size) = event {
                    renderer.resize(new_size);
                }

                if let WindowEvent::RedrawRequested = event {
                    app.tick();
                    if let Err(err) = renderer.render(&app.state) {
                        error!("render failed: {err:#}");
                    }
                    last_redraw = Instant::now();
                } else {
                    for action in input.map_window_event(&event) {
                        app.apply_action(action);
                    }
                    app.tick();
                }
            }
            Event::AboutToWait => {
                let elapsed = last_redraw.elapsed().as_millis();
                if elapsed > 16 {
                    window.request_redraw();
                }
            }
            Event::LoopExiting => info!("spatial file browser exited"),
            _ => {}
        }
    })?;

    #[allow(unreachable_code)]
    Ok(())
}
