use crate::*;
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
    tx_dist: Waker,
}

impl GraphicsCaptureApiHandler for Streamer {
    type Error = Error;
    type Flags = (Arc<Mutex<Region>>, Arc<AtomicU8>, Arc<AtomicBool>, Waker);

    // Function that will be called to create a new instance. The flags can be
    // passed from settings.
    fn new(ctx: windows_capture::capture::Context<Self::Flags>) -> Result<Self, Self::Error> {
        let (region, region_id, is_running, tx_dist) = ctx.flags;
        Ok(Self {
            region,
            region_id,
            is_running,
            tx_dist,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut windows_capture::frame::Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        if !self.is_running.load(Ordering::Relaxed) {
            capture_control.stop();
            self.tx_dist.signal();
            return Ok(());
        }

        let mut frame_buffer = frame.buffer()?;
        let (w, h) = (frame_buffer.width() as i32, frame_buffer.height() as i32);
        let src = frame_buffer.as_raw_buffer();
        let l = src.len();

        // TODO - impl regional tiling

        {
            let mut region = self.region.lock();

            // update region metadata and buffer
            region.set_x(0);
            region.set_y(0);
            region.set_w(w);
            region.set_h(h);
            region.set_l(l);

            let dst = region.data_mut();

            if dst.capacity() < l {
                dst.reserve_exact(l - dst.capacity());
            }
            unsafe {
                dst.set_len(l);
                std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), l);
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
        self.tx_dist.wake();
        Ok(())
    }
}

pub fn start_capturing(
    cfg: Config,
    region: Arc<Mutex<Region>>,
    region_id: Arc<AtomicU8>,
    is_running: Arc<AtomicBool>,
    (tx_dist, rx_conn): (Waker, Waiter),
) -> JoinHandle<Result<()>> {
    spawn(move || {
        rx_conn.update_thread();

        // Gets the primary monitor, refer to the docs for other capture items.
        let target = Window::from_contains_name(cfg.window())?;

        loop {
            // wait for connection
            rx_conn.wait();

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
                    tx_dist.clone(),
                ),
            );

            // begin screen capturing
            if let Err(e) = Streamer::start(settings) {
                eprintln!("run: {:?}", e);
                // fallback reset
                is_running.store(false, Ordering::SeqCst);
            }
        }
    })
}
