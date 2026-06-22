use std::num::NonZeroU32;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::prelude::GlSurface;
use glutin::surface::{Surface, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
#[cfg(target_os = "macos")]
use winit::platform::macos::{OptionAsAlt, WindowAttributesExtMacOS};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{Cursor, CursorGrabMode, CursorIcon, Fullscreen, Icon, Theme, Window, WindowButtons, WindowId, WindowLevel};
use crate::color::Color;

pub trait VitreousRSHandler{
    fn init(&self);
    fn render(&self);
    fn exit(&self);
}

pub struct WindowData {
    pub title: String,
    pub title_hidden: bool,
    #[cfg(target_os = "macos")]
    pub titlebar_hidden: bool,
    #[cfg(target_os = "macos")]
    pub titlebar_buttons_hidden: bool,
    #[cfg(target_os = "macos")]
    pub titlebar_transparent: bool,
    #[cfg(target_os = "macos")]
    pub fullsize_content_view: bool,
    #[cfg(target_os = "macos")]
    pub has_shadow: bool,
    pub blur: bool,
    pub window_mode: WindowMode,
    pub position: Position,
    pub size: Size,
    pub min_inner_size: Size,
    pub max_inner_size: Size,
    pub decorations: bool,
    pub transparent: bool,
    pub resizable: bool,
    pub resize_increments: Size,
    pub maximized: bool,
    pub visible: bool,
    pub always_on_top: bool,
    pub window_icon: Option<Icon>,
    pub cursor: Cursor,
    pub enabled_buttons: WindowButtons,
    pub active: bool,
    pub window_level: WindowLevel,
    #[cfg(target_os = "macos")]
    pub accepts_first_mouse: bool,
    #[cfg(target_os = "macos")]
    pub option_as_alt: OptionAsAlt,
    #[cfg(target_os = "macos")]
    pub disallow_hidpi: bool,
    pub content_protected: bool,
    pub parent_window: Option<RawWindowHandle>,
    #[cfg(target_os = "macos")]
    pub tabbing_identifier: String,
    pub movable_by_window_background: bool,
    pub theme: Option<Theme>,
    pub clear_color: Color,
}

impl Default for WindowData {
    fn default() -> Self {
        Self {
            title: String::from("VitreousRS Window"),
            title_hidden: false,
            #[cfg(target_os = "macos")]
            titlebar_hidden: false,
            #[cfg(target_os = "macos")]
            titlebar_buttons_hidden: false,
            #[cfg(target_os = "macos")]
            titlebar_transparent: false,
            #[cfg(target_os = "macos")]
            fullsize_content_view: false,
            #[cfg(target_os = "macos")]
            has_shadow: false,
            blur: false,
            window_mode: WindowMode::WINDOW,
            position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
            size: Size::Logical(LogicalSize::new(1024.0, 768.0)),
            min_inner_size: Size::Logical(LogicalSize::new(64.0,64.0)),
            max_inner_size: Size::Logical(LogicalSize::new(16384.0, 16384.0)),
            decorations: true,
            transparent: false,
            resizable: true,
            resize_increments: Size::Physical(PhysicalSize::new(1,1)),
            maximized: false,
            visible: true,
            always_on_top: false,
            window_icon: None,
            cursor: Cursor::default(),
            enabled_buttons: WindowButtons::all(),
            active: false,
            window_level: WindowLevel::default(),
            #[cfg(target_os = "macos")]
            accepts_first_mouse: true,
            #[cfg(target_os = "macos")]
            option_as_alt: OptionAsAlt::default(),
            #[cfg(target_os = "macos")]
            disallow_hidpi: false,
            content_protected: false,
            parent_window: None,
            #[cfg(target_os = "macos")]
            tabbing_identifier: String::from(""),
            movable_by_window_background: false,
            theme: None,
            clear_color: Color::default()
        }
    }
}

enum WindowMode{
    WINDOW,
    BORDERLESS,
    FULLSCREEN,
}

pub struct Application<H : VitreousRSHandler> {
    pub window_data: WindowData,
    pub control_flow: ControlFlow,
    pub vitreous_rs_handler: H,
    window: Option<Window>,
    gl_surface: Option<Surface<WindowSurface>>,
    gl_context: Option<PossiblyCurrentContext>,
}

impl <H : VitreousRSHandler> Application<H> {
    pub fn new(window_data: WindowData, control_flow: ControlFlow, vitreous_rs_handler: H) -> Self {
        Self {
            window_data,
            control_flow,
            vitreous_rs_handler,
            window: None,
            gl_surface: None,
            gl_context: None,
        }
    }

    pub fn run(&mut self){
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(self.control_flow);
        event_loop.run_app(self).unwrap();
    }
}

impl<H : VitreousRSHandler> ApplicationHandler for Application<H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let fullscreen_setting = match self.window_data.window_mode {
            WindowMode::WINDOW => None,
            WindowMode::BORDERLESS => Some(Fullscreen::Borderless(None)),
            WindowMode::FULLSCREEN => {
                if let Some(monitor) = event_loop.primary_monitor() {
                    monitor.video_modes().next().map(Fullscreen::Exclusive)
                } else {
                    Some(Fullscreen::Borderless(None))
                }
            }
        };

        let mut window_attributes = Window::default_attributes()
            .with_title(self.window_data.title.clone())
            .with_active(self.window_data.active)
            .with_blur(self.window_data.blur)
            .with_cursor(self.window_data.cursor.clone())
            .with_content_protected(self.window_data.content_protected)
            .with_decorations(self.window_data.decorations)
            .with_enabled_buttons(self.window_data.enabled_buttons)
            .with_fullscreen(fullscreen_setting)
            .with_inner_size(self.window_data.size)
            .with_maximized(self.window_data.maximized)
            .with_min_inner_size(self.window_data.min_inner_size)
            .with_max_inner_size(self.window_data.max_inner_size)
            .with_position(self.window_data.position)
            .with_resizable(self.window_data.resizable)
            .with_theme(self.window_data.theme)
            .with_transparent(self.window_data.transparent)
            .with_visible(self.window_data.visible)
            .with_window_icon(self.window_data.window_icon.clone())
            .with_window_level(self.window_data.window_level);

        match self.window_data.resize_increments {
            Size::Physical(p_size) => {
                if p_size.width > 1 || p_size.height > 1 {
                    window_attributes = window_attributes.with_resize_increments(self.window_data.resize_increments);
                }
            }
            Size::Logical(l_size) => {
                if l_size.width > 1.0 || l_size.height > 1.0 {
                    window_attributes = window_attributes.with_resize_increments(self.window_data.resize_increments);
                }
            }
        }

        if let Some(parent_handle) = self.window_data.parent_window {
            unsafe {
                window_attributes = window_attributes.with_parent_window(Some(parent_handle));
            }
        }

        #[cfg(target_os = "macos")]
        {
            window_attributes = window_attributes
                .with_titlebar_hidden(self.window_data.titlebar_hidden)
                .with_titlebar_buttons_hidden(self.window_data.titlebar_buttons_hidden)
                .with_titlebar_transparent(self.window_data.titlebar_transparent)
                .with_fullsize_content_view(self.window_data.fullsize_content_view)
                .with_accepts_first_mouse(self.window_data.accepts_first_mouse)
                .with_option_as_alt(self.window_data.option_as_alt)
                .with_tabbing_identifier(&self.window_data.tabbing_identifier)
                .with_movable_by_window_background(self.window_data.movable_by_window_background)
                .with_title_hidden(self.window_data.title_hidden)
                .with_borderless_game(match self.window_data.window_mode {
                    WindowMode::WINDOW => false,
                    WindowMode::BORDERLESS => true,
                    WindowMode::FULLSCREEN => false,
                })
                .with_disallow_hidpi(self.window_data.disallow_hidpi)
                .with_has_shadow(self.window_data.has_shadow);
        }

        let template = ConfigTemplateBuilder::new();
        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));
        let (window, gl_config) = display_builder
            .build(
                event_loop,
                template,
                |mut configs| configs.next().unwrap())
            .unwrap();

        let window = window.unwrap();
        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version { major: 4, minor: 6 })))
            .with_profile(glutin::context::GlProfile::Core)
            .build(Some(window.window_handle().unwrap().as_raw()));

        let not_current_gl_context = unsafe {
            gl_display.create_context(&gl_config, &context_attributes).unwrap()
        };

        let attrs = window.build_surface_attributes(Default::default()).unwrap();
        let gl_surface = unsafe {
            gl_config.display().create_window_surface(&gl_config, &attrs).unwrap()
        };
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        gl::load_with(|symbol| {
            let symbol = std::ffi::CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        crate::draw_call::load_extensions(|name| {
            let c_name = std::ffi::CString::new(name).unwrap();
            gl_display.get_proc_address(c_name.as_c_str()).cast()
        });

        self.window = Some(window);
        self.gl_surface = Some(gl_surface);
        self.gl_context = Some(gl_context);

        self.window.as_ref().unwrap().request_redraw();

        self.vitreous_rs_handler.init();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.vitreous_rs_handler.exit();
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width != 0 && size.height != 0 {
                    if let (Some(surface), Some(context)) = (&self.gl_surface, &self.gl_context) {
                        surface.resize(context, NonZeroU32::new(size.width).unwrap(), NonZeroU32::new(size.height).unwrap());
                        unsafe {
                            gl::Viewport(0, 0, size.width as i32, size.height as i32);
                        }
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                if let (Some(surface), Some(context), Some(window)) = (&self.gl_surface, &self.gl_context, &self.window) {
                    unsafe {
                        gl::ClearColor(self.window_data.clear_color.red(), self.window_data.clear_color.green(), self.window_data.clear_color.blue(), self.window_data.clear_color.alpha());
                        gl::Clear(gl::COLOR_BUFFER_BIT);
                    }

                    self.vitreous_rs_handler.render();

                    surface.swap_buffers(context).unwrap();
                    window.request_redraw();
                }
            }
            _ => ()
        }
    }
}