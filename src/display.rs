use crate::config;
use xcb::{self, randr};

pub struct Display {
    connection: xcb::Connection,
    window: u32,
    gc: u32,
    screen_index: usize,
    screen_resources: Option<randr::GetScreenResourcesReply>,
    previous_screen: Option<ScreenInfo>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
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
            screen_resources: None,
            previous_screen: None,
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

    fn get_size_and_offset(&mut self) -> ScreenInfo {
        let Display {
            screen_resources,
            connection,
            window,
            ..
        } = self;
        let screen_resources = screen_resources.get_or_insert_with(|| {
            let sr_cookie = xcb::randr::get_screen_resources(&connection, *window);
            sr_cookie
                .get_reply()
                .expect("Could not get screen resources.")
        });
        let pointer_cookie = xcb::query_pointer(&self.connection, *window);
        let pointer_reply = pointer_cookie
            .get_reply()
            .expect("Could not get pointer position.");

        let x = pointer_reply.root_x();
        let y = pointer_reply.root_y();

        let crtcs = screen_resources.crtcs();
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

    fn configure_window(&mut self, screen_info: &ScreenInfo, global_config: &config::GlobalConfig) {
        if self
            .previous_screen
            .map(|prev| &prev == screen_info)
            .unwrap_or(false)
        {
            return;
        }
        self.previous_screen = Some(*screen_info);

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
    }

    fn draw_rectangle(&self, color: u32, rectangle: xcb::Rectangle) {
        xcb::change_gc(&self.connection, self.gc, &[(xcb::GC_FOREGROUND, color)]);
        xcb::poly_fill_rectangle(&self.connection, self.window, self.gc, &[rectangle]);
    }

    fn draw_bar(
        &self,
        value: u8,
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

        let height_diff =
            f64::from(global_config.height(screen_info.height)) * (100 - value) as f64 / 100.0;
        let width_diff =
            f64::from(global_config.width(screen_info.width)) * (100 - value) as f64 / 100.0;

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
        &mut self,
        value: u8,
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
