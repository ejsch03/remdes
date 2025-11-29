use crate::base::Config;
use anyhow::*;
use parking_lot::Mutex;
use remdes::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::thread::{JoinHandle, spawn};
use windows_capture::window::Window;
use windows_capture::{
    capture::GraphicsCaptureApiHandler,
    graphics_capture_api::InternalCaptureControl,
    settings::Settings,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings,
    },
};

struct Streamer {
    region: Arc<Mutex<Region>>,
    region_id: Arc<AtomicU8>,
    is_running: Arc<AtomicBool>,
    tx: waitx::Waker,
}

impl GraphicsCaptureApiHandler for Streamer {
    type Error = Error;
    type Flags = (
        Arc<Mutex<Region>>,
        Arc<AtomicU8>,
        Arc<AtomicBool>,
        waitx::Waker,
    );

    // Function that will be called to create a new instance. The flags can be
    // passed from settings.
    fn new(ctx: windows_capture::capture::Context<Self::Flags>) -> Result<Self, Self::Error> {
        let (region, region_id, is_running, tx) = ctx.flags;
        Ok(Self {
            region,
            region_id,
            is_running,
            tx,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut windows_capture::frame::Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        if !self.is_running.load(Ordering::Relaxed) {
            capture_control.stop();
            bail!("Stopping.")
        }

        let mut frame_buffer = frame.buffer()?;
        let (w, h) = (frame_buffer.width() as i32, frame_buffer.height() as i32);
        let src = frame_buffer.as_raw_buffer();
        let l = src.len();

        // TODO - locate differences?

        {
            let mut region = self.region.lock();

            // update region metadata and buffer
            region.set_x(0);
            region.set_y(0);
            region.set_w(w);
            region.set_h(h);
            region.set_l(l);

            let dst = region.data_mut();

            if dst.capacity() < src.len() {
                dst.reserve_exact(src.len() - dst.capacity());
            }
            unsafe {
                dst.set_len(src.len());
                std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
            }
        }

        // SAFETY - fetch_update always returns Some(..)
        unsafe {
            self.region_id
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |id| {
                    Some(id.wrapping_add(1))
                })
                .unwrap_unchecked();
        }
        self.tx.wake();
        Ok(())
    }
}

pub fn start_capturing(
    cfg: Config,
    region: Arc<Mutex<Region>>,
    region_id: Arc<AtomicU8>,
    is_running: Arc<AtomicBool>,
    tx: waitx::Waker,
) -> JoinHandle<Result<()>> {
    spawn(move || {
        // Gets the primary monitor, refer to the docs for other capture items.
        let target = Window::from_contains_name(cfg.window())?;

        loop {
            // reset back to normal state (true)
            is_running.store(true, Ordering::SeqCst);

            // these settings not necessarily universally compatible
            // TODO - figure out minimum compatible defaults.
            let settings = Settings::new(
                target,
                CursorCaptureSettings::Default,
                DrawBorderSettings::WithoutBorder,
                SecondaryWindowSettings::Default,
                MinimumUpdateIntervalSettings::Custom(cfg.tps()),
                DirtyRegionSettings::ReportOnly,
                ColorFormat::Bgra8,
                (
                    region.clone(),
                    region_id.clone(),
                    is_running.clone(),
                    tx.clone(),
                ),
            );

            // begin screen capturing
            if let Err(e) = Streamer::start(settings) {
                eprintln!("run: {}", e);
                break Ok(());
            }

            // this lets the windows-capture thread know when to exit
            is_running.store(false, Ordering::SeqCst);
        }
    })
}
