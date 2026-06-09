//! Criação do contexto OpenGL com glutin 0.32 + winit 0.30.

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentContext, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use std::ffi::CString;
use std::num::NonZeroU32;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::HasWindowHandle;
use winit::window::Window;

/// Contexto OpenGL completo.
pub struct GlContext {
    pub surface: Surface<WindowSurface>,
    pub context: glutin::context::PossiblyCurrentContext,
    pub gl_config: glutin::config::Config,
}

impl GlContext {
    /// Cria janela + contexto OpenGL 3.3 Core.
    pub fn new(
        event_loop: &ActiveEventLoop,
        title: &str,
        width: u32,
        height: u32,
    ) -> Result<(Window, Self), String> {
        use winit::dpi::LogicalSize;

        let window_attrs = Window::default_attributes()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width as f64, height as f64));

        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_depth_size(24);

        let (window, gl_config) = DisplayBuilder::new()
            .with_window_attributes(Some(window_attrs))
            .build(
                event_loop,
                template,
                |configs| {
                    configs
                        .filter(|c| c.depth_size() > 0)
                        .min_by_key(|c| c.num_samples())
                        .unwrap()
                },
            )
            .map_err(|e| e.to_string())?;

        let window = window.ok_or("Falha ao criar janela")?;
        let gl_display = gl_config.display();

        let raw = window
            .window_handle()
            .map_err(|e| e.to_string())?
            .as_raw();

        let ctx_attrs = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(raw));

        let (w, h) = (
            NonZeroU32::new(window.inner_size().width.max(1)).unwrap(),
            NonZeroU32::new(window.inner_size().height.max(1)).unwrap(),
        );

        let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new()
            .build(raw, w, h);

        let (context, surface) = unsafe {
            let context: NotCurrentContext = gl_display
                .create_context(&gl_config, &ctx_attrs)
                .map_err(|e| e.to_string())?;

            let surface = gl_display
                .create_window_surface(&gl_config, &surface_attrs)
                .map_err(|e| e.to_string())?;

            (context, surface)
        };

        let context = context.make_current(&surface).map_err(|e| e.to_string())?;

        surface
            .set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
            .ok();

        Ok((
            window,
            GlContext {
                surface,
                context,
                gl_config,
            },
        ))
    }

    pub fn load_gl(&self) {
        let gl_display = self.gl_config.display();
        gl::load_with(|symbol| {
            let name = CString::new(symbol).expect("CString::new failed");
            gl_display.get_proc_address(name.as_c_str()) as *const _
        });
    }

    pub fn resize(&self, width: u32, height: u32) {
        if let (Some(w), Some(h)) = (
            NonZeroU32::new(width.max(1)),
            NonZeroU32::new(height.max(1)),
        ) {
            let _ = self.surface.resize(&self.context, w, h);
        }
    }

    pub fn swap_buffers(&self) -> Result<(), String> {
        self.surface
            .swap_buffers(&self.context)
            .map_err(|e| e.to_string())
    }
}
