use ash::extensions::khr::Surface;
use ash::*;
use graphyte_engine::*;

pub(crate) struct WindowRenderTarget {
    window: PlatformWindowHandle,
    surface: vk::SurfaceKHR,
    surface_fn: Surface,
}

impl WindowRenderTarget {
    pub fn new(entry: &Entry, instance: &Instance, window: &dyn PlatformWindow) -> Option<Self> {
        let surface = unsafe {
            ash_window::create_surface(entry, instance, &window.raw_platform_handle(), None)
                .ok()?
                .into()
        };
        let surface_fn = ash::extensions::khr::Surface::new(entry, instance);
        Self {
            window: window.handle(),
            surface,
            surface_fn,
        }
        .into()
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub fn window(&self) -> PlatformWindowHandle {
        self.window
    }
}

impl Drop for WindowRenderTarget {
    fn drop(&mut self) {
        unsafe {
            self.surface_fn.destroy_surface(self.surface, None);
        }
    }
}
