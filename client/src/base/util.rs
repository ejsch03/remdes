use crate::*;

pub fn init() -> Result<(
    sdl2::Sdl,
    sdl2::VideoSubsystem,
    sdl2::video::Window,
    sdl2::EventSubsystem,
    sdl2::EventPump,
    sdl2::video::GLContext,
    glow::Context,
)> {
    let sdl = sdl2::init().map_err(|e| anyhow!(e))?;
    let video = sdl.video().map_err(|e| anyhow!(e))?;

    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 6);

    let (width, height) = video.display_bounds(0).map_err(|e| anyhow!(e))?.size();

    let window = video
        .window(
            "remdes",
            (width as f32 / 1.4) as u32,
            (height as f32 / 1.4) as u32,
        )
        .resizable()
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| anyhow!(e))?;

    // Create the GL context (makes it current for this thread)
    let gl_context = window.gl_create_context().map_err(|e| anyhow!(e))?;

    // immediate mode with relative mouse mode
    video
        .gl_set_swap_interval(SwapInterval::Immediate)
        .map_err(|e| anyhow!(e))?;
    sdl.mouse().set_relative_mouse_mode(true);

    let ev = sdl.event().map_err(|e| anyhow!(e))?;
    let ep = sdl.event_pump().map_err(|e| anyhow!(e))?;

    // Register custom event type for payloads
    ev.register_custom_event::<UserEvent>()
        .map_err(|e| anyhow!(e))?;

    let mut gl = unsafe {
        glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _)
    };

    unsafe {
        // alpha blending
        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

        #[cfg(debug_assertions)]
        {
            gl.enable(glow::DEBUG_OUTPUT);
            gl.debug_message_callback(|source, ty, id, severity, msg| match severity {
                glow::DEBUG_SEVERITY_LOW => {
                    log::info!("GL {{ src: {source}, ty: {ty}, id: {id} }} {msg}")
                }
                glow::DEBUG_SEVERITY_MEDIUM => {
                    log::warn!("GL {{ src: {source}, ty: {ty}, id: {id} }} {msg}")
                }
                glow::DEBUG_SEVERITY_HIGH => {
                    log::error!("GL {{ src: {source}, ty: {ty}, id: {id} }} {msg}")
                }
                glow::DEBUG_SEVERITY_NOTIFICATION => {}
                _ => log::debug!("GL::UNKNOWN {{ src: {source}, ty: {ty}, id: {id} }} {msg}"),
            });
        }
    }

    Ok((sdl, video, window, ev, ep, gl_context, gl))
}
