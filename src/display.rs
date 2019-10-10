use crate::config;
use xcb;

pub struct Display {
    connection: xcb::Connection,
    window: u32,
    gc: u32,
    screen_index: usize,
}

struct ScreenInfo {
    width: u32,
    height: u32,
    x: i16,
    y: i16,
}

impl Display {
    fn init_window(&self) {
        let screen = self
            .connection
            .get_setup()
            .roots()
            .nth(self.screen_index)
            .expect("Expected screen to exist.");

        xcb::create_window(
            &self.connection,
            xcb::COPY_FROM_PARENT as u8,
            self.window,
            screen.root(),
            0,
            0,
            1,
            1,
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            screen.root_visual(),
            &[(xcb::CW_OVERRIDE_REDIRECT, 1)],
        );
    }

    fn init_gc(&self) {
        let screen = self
            .connection
            .get_setup()
            .roots()
            .nth(self.screen_index)
            .expect("Expected screen to exist.");

        xcb::create_gc(&self.connection, self.gc, screen.root(), &[]);
    }

    pub fn new() -> Result<Self, xcb::ConnError> {
        let (connection, screen_index) = xcb::Connection::connect(None)?;
        let screen_index = screen_index as usize;
        let window = connection.generate_id();
        let gc = connection.generate_id();

        let ret = Display {
            connection,
            window,
            gc,
            screen_index,
        };

        ret.init_window();
        ret.init_gc();

        let title = env!("CARGO_PKG_NAME");
        xcb::change_property(
            &ret.connection,
            xcb::PROP_MODE_REPLACE as u8,
            ret.window,
            xcb::ATOM_WM_NAME,
            xcb::ATOM_STRING,
            8,
            title.as_bytes(),
        );

        xcb::change_property(
            &ret.connection,
            xcb::PROP_MODE_REPLACE as u8,
            ret.window,
            xcb::ATOM_WM_CLASS,
            xcb::ATOM_STRING,
            8,
            title.as_bytes(),
        );

        ret.connection.flush();

        Ok(ret)
    }

    fn get_size_and_offset(&self) -> ScreenInfo {
        let dummy_window = self.connection.generate_id();
        let screen = self
            .connection
            .get_setup()
            .roots()
            .nth(self.screen_index)
            .expect("Expected screen to exist.");

        xcb::create_window(
            &self.connection,
            0,
            dummy_window,
            screen.root(),
            0,
            0,
            1,
            1,
            0,
            0,
            0,
            &[],
        );

        self.connection.flush();

        let sr_cookie = xcb::randr::get_screen_resources(&self.connection, dummy_window);
        let sr_reply = sr_cookie
            .get_reply()
            .expect("Could not get screen resources.");
        let pointer_cookie = xcb::query_pointer(&self.connection, dummy_window);
        let pointer_reply = pointer_cookie
            .get_reply()
            .expect("Could not get pointer position.");
        xcb::destroy_window(&self.connection, dummy_window);

        let x = pointer_reply.root_x();
        let y = pointer_reply.root_y();
        let crtcs = sr_reply.crtcs();
        for crtc in crtcs {
            let crtc_cookie = xcb::randr::get_crtc_info(&self.connection, *crtc, 0);
            if let Ok(reply) = crtc_cookie.get_reply() {
                if reply.x() <= x
                    && x < reply.x() + reply.width() as i16
                    && reply.y() <= y
                    && y < reply.y() + reply.height() as i16
                {
                    return ScreenInfo {
                        width: u32::from(reply.width()),
                        height: u32::from(reply.height()),
                        x: reply.x(),
                        y: reply.y(),
                    };
                }
            }
        }

        panic!("Pointer location was not on any screen.");
    }

    fn configure_window(&self, screen_info: &ScreenInfo, global_config: &config::GlobalConfig) {
        let width = global_config.total_width(screen_info.width);
        let height = global_config.total_height(screen_info.height);
        let x = global_config.x(screen_info.width) + screen_info.x as u32;
        let y = global_config.y(screen_info.height) + screen_info.y as u32;

        xcb::configure_window(
            &self.connection,
            self.window,
            &[
                (xcb::CONFIG_WINDOW_WIDTH as u16, width),
                (xcb::CONFIG_WINDOW_HEIGHT as u16, height),
                (xcb::CONFIG_WINDOW_X as u16, x),
                (xcb::CONFIG_WINDOW_Y as u16, y),
                (xcb::CONFIG_WINDOW_STACK_MODE as u16, xcb::STACK_MODE_ABOVE),
            ],
        );
        self.connection.flush();
    }

    fn draw_rectangle(&self, color: u32, rectangle: xcb::Rectangle) {
        xcb::change_gc(&self.connection, self.gc, &[(xcb::GC_FOREGROUND, color)]);
        xcb::poly_fill_rectangle(&self.connection, self.window, self.gc, &[rectangle]);
    }

    fn draw_bar(
        &self,
        value: f64,
        screen_info: &ScreenInfo,
        global_config: &config::GlobalConfig,
        color_config: &config::ColorConfig,
    ) {
        let mut x = 0;
        let mut y = 0;
        let mut width = global_config.total_width(screen_info.width) as u16;
        let mut height = global_config.total_height(screen_info.height) as u16;
        self.draw_rectangle(
            color_config.background,
            xcb::Rectangle::new(x, y, width, height),
        );

        x += global_config.margin as i16;
        y += global_config.margin as i16;
        width -= global_config.margin as u16 * 2;
        height -= global_config.margin as u16 * 2;
        self.draw_rectangle(
            color_config.border,
            xcb::Rectangle::new(x, y, width, height),
        );

        x += global_config.border as i16;
        y += global_config.border as i16;
        width -= global_config.border as u16 * 2;
        height -= global_config.border as u16 * 2;
        self.draw_rectangle(
            color_config.background,
            xcb::Rectangle::new(x, y, width, height),
        );

        let height_diff = f64::from(global_config.height(screen_info.height)) * (1.0 - value);
        let width_diff = f64::from(global_config.width(screen_info.width)) * (1.0 - value);

        x += global_config.padding as i16;
        y += global_config.padding as i16;
        width -= global_config.padding as u16 * 2;
        height -= global_config.padding as u16 * 2;

        match global_config.fill_direction {
            config::Direction::Up => {
                y += height_diff as i16;
                height -= height_diff as u16;
            }
            config::Direction::Down => {
                height -= height_diff as u16;
            }
            config::Direction::Left => {
                x += width_diff as i16;
                width -= width_diff as u16;
            }
            config::Direction::Right => {
                width -= width_diff as u16;
            }
        }

        self.draw_rectangle(
            color_config.foreground,
            xcb::Rectangle::new(x, y, width, height),
        );
    }

    pub fn show(
        &self,
        value: f64,
        global_config: &config::GlobalConfig,
        color_config: &config::ColorConfig,
    ) {
        let screen_info = self.get_size_and_offset();
        xcb::map_window(&self.connection, self.window);
        self.configure_window(&screen_info, global_config);
        self.draw_bar(value, &screen_info, global_config, color_config);
        self.connection.flush();
    }

    pub fn hide(&self) {
        xcb::unmap_window(&self.connection, self.window);
        self.connection.flush();
    }
}
