use crate::*;
use std::sync::atomic::AtomicU64;

#[derive(Debug, Default)]
pub struct FpsUpdater {
    inner: AtomicU8, // incrementer
}

impl FpsUpdater {
    pub fn incr(&self) {
        let _ = self
            .inner
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
                Some(val.saturating_add(1))
            });
    }

    pub fn upt(&self, rhs: &Fps) {
        let fps = self.inner.swap(0, Ordering::Relaxed);
        rhs.inner.store(fps, Ordering::Relaxed);
    }
}

#[derive(Debug, Default)]
pub struct Fps {
    inner: AtomicU8, // frames-per-second
}

impl Fps {
    pub fn get(&self) -> u8 {
        self.inner.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Default)]
pub struct Limit {
    nanos: AtomicU64,
}

impl Limit {
    pub fn get(&self) -> Duration {
        Duration::from_nanos(self.nanos.load(Ordering::Relaxed))
    }

    pub fn set(&self, fps: u8) {
        let dur = remdes::util::tick_dur(fps as f32);
        let nanos = dur.as_nanos().min(u64::MAX as u128) as u64;
        self.nanos.store(nanos, Ordering::Relaxed);
    }
}

pub fn init_fps_display(fps: Arc<Fps>) -> JoinHandle<Result<()>> {
    spawn(move || {
        let spinner = SpinSleeper::default();
        let mut out = std::io::stdout();

        loop {
            out.write_all(format!("\x1b[2J\x1b[H{}\n", fps.get()).as_bytes())
                .unwrap();
            out.flush().unwrap();
            spinner.sleep(SECOND);
        }
    })
}

pub fn set_fps_limit_fn(
    limit: Arc<AtomicU8>,
    limit_dur: Arc<Limit>,
    state: Arc<AtomicRenderStateKind>,
) -> impl Fn(u8) {
    move |target| {
        // old and new rendering states
        let old_state = limit_dur.get().is_zero();
        let new_state = target == 0;

        // only reload the rendering state if fps polygon_mode is changed
        if old_state != new_state {
            state.store(RenderStateKind::Reload, Ordering::Relaxed);
        }

        // update limiter
        limit.store(target, Ordering::Relaxed);
        limit_dur.set(target);
    }
}

pub fn init_fps_resetter(fps: Arc<Fps>, fps_upt: Arc<FpsUpdater>) -> JoinHandle<()> {
    spawn(move || {
        let spin = SpinSleeper::default();
        loop {
            spin.sleep(SECOND);
            fps_upt.upt(&fps);
        }
    })
}
