#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod base;
use base::*;

mod generated;
use generated::*;

use anyhow::*;
use atomic_enum::*;
use glow::HasContext;
use parking_lot::Mutex;
use remdes::*;
use sdl2::{event::*, keyboard::*, video::*};
use spin_sleep::SpinSleeper;
use std::{
    io::Write,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    thread::{JoinHandle, spawn},
    time::Duration,
};
use waitx::*;

/// Current state of the rendering thread.
#[atomic_enum]
#[derive(Default)]
pub enum RenderStateKind {
    #[default]
    Pass,
    Reload,
    Quit,
}

#[derive(Clone, Copy, Debug)]
pub enum UserEvent {
    Render,
    Fps(u8),
}

/// Render the texture
fn display(gl: &glow::Context, window: &Window) {
    unsafe {
        gl.clear(glow::COLOR_BUFFER_BIT);
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        window.gl_swap_window();

        #[cfg(debug_assertions)]
        gl.finish(); // benchmarking
    }
}

/// Event loop: handles new textures and updates VBO with scale
fn event_loop(
    gl: &glow::Context,
    window: Window,
    mut ep: sdl2::EventPump,
    tex: &mut Texture2D,
    frame: Arc<Mutex<Region>>,
    tx_render: Waker,
    set_fps_limit: &mut impl FnMut(u8),
) {
    for event in ep.wait_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => break,

            Event::Window {
                win_event: WindowEvent::Resized(w, h),
                ..
            } => {
                unsafe { gl.viewport(0, 0, w, h) };
            }

            Event::User { .. } => {
                let ue = event.as_user_event_type::<UserEvent>().unwrap();

                match ue {
                    UserEvent::Render => {
                        tex.update(gl, &frame.lock());
                        display(gl, &window);
                        tx_render.signal();
                    }

                    // set a new fps target
                    UserEvent::Fps(fps) => set_fps_limit(fps),
                }
            }
            _ => (),
        }
    }
}

fn main() -> Result<()> {
    let cfg = Config::default();
    env_logger::init();

    // Setup SDL and OpenGL
    let (_sdl, _video, window, ev, ep, _ctx, gl) = init()?;

    // Initialize shader and texture
    let progs = Shaders::init(&gl)?;

    // frames-per-second facilitation
    let (fps, fps_upt, limit, limit_dur): (Arc<Fps>, Arc<FpsUpdater>, Arc<AtomicU8>, Arc<Limit>) =
        Default::default();

    // determinant of the status of threads
    let state = Arc::new(AtomicRenderStateKind::new(RenderStateKind::default()));

    // reset the fps counter every second
    let _fps_resetter = init_fps_resetter(fps.clone(), fps_upt.clone());

    // display fps via terminal (stdout)
    let _fps_display = init_fps_display(fps);

    // helper closure for settings the fps limit
    let mut set_fps_limit = set_fps_limit_fn(limit, limit_dur.clone(), state);
    set_fps_limit(cfg.fps()); // initial fps configuration

    // frame data communication
    let (tx_render, rx_render) = pair();
    let frame: Arc<Mutex<Region>> = Default::default();

    // networking thread
    let _conn = init_remote(
        cfg,
        ev.event_sender(),
        frame.clone(),
        rx_render,
        fps_upt,
        limit_dur,
    );

    // texture for frame data
    let mut tex = Texture2D::new(&gl);
    unsafe {
        gl.use_program(Some(progs.simple().native()));
        gl.bind_vertex_array(Some(tex.vao));
    }

    // start polling input and custom events
    event_loop(
        &gl,
        window,
        ep,
        &mut tex,
        frame,
        tx_render,
        &mut set_fps_limit,
    );

    // clean up
    progs.delete(&gl);
    tex.delete(&gl);

    Ok(())
}
