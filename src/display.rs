use config;
use xcb;

pub struct Display {
    connection: xcb::Connection,
    window: u32,
    gc: u32,
    screen_index: usize,
}

impl Display {
    fn init_window(&self) {
        let screen = self.connection
            .get_setup()
            .roots()
            .nth(self.screen_index)
            .expect("Expected screen to exist.");

        xcb::create_window(
            &self.connection,
            xcb::COPY_FROM_PARENT as u8,
            self.window,
            screen.root(),
            0, 0,
            1, 1,
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            screen.root_visual(),
            &[(xcb::CW_OVERRIDE_REDIRECT, 1)]
        );
    }

    fn init_gc(&self) {
        let screen = self.connection
            .get_setup()
            .roots()
            .nth(self.screen_index)
            .expect("Expected screen to exist.");

        xcb::create_gc(
            &self.connection,
            self.gc,
            screen.root(),
            &[],
        );
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
            title.as_bytes()
        );

        xcb::change_property(
            &ret.connection,
            xcb::PROP_MODE_REPLACE as u8,
            ret.window,
            xcb::ATOM_WM_CLASS,
            xcb::ATOM_STRING,
            8,
            title.as_bytes()
        );

        ret.connection.flush();

        Ok(ret)
    }

    fn configure_window(&self, global_config: &config::GlobalConfig) {
        xcb::configure_window(
            &self.connection,
            self.window,
            &[
                (xcb::CONFIG_WINDOW_WIDTH as u16, global_config.width_to_margin()),
                (xcb::CONFIG_WINDOW_HEIGHT as u16, global_config.height_to_margin()),
                (xcb::CONFIG_WINDOW_X as u16, global_config.x()),
                (xcb::CONFIG_WINDOW_Y as u16, global_config.y()),
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
        global_config: &config::GlobalConfig,
        color_config: &config::ColorConfig,
    ) {
        let mut x = 0;
        let mut y = 0;
        let mut width = global_config.width_to_margin() as u16;
        let mut height = global_config.height_to_margin() as u16;
        self.draw_rectangle(color_config.background, xcb::Rectangle::new(x, y, width, height));

        x += global_config.margin as i16;
        y += global_config.margin as i16;
        width -= global_config.margin as u16 * 2;
        height -= global_config.margin as u16 * 2;
        self.draw_rectangle(color_config.border, xcb::Rectangle::new(x, y, width, height));

        x += global_config.border as i16;
        y += global_config.border as i16;
        width -= global_config.border as u16 * 2;
        height -= global_config.border as u16 * 2;
        self.draw_rectangle(color_config.background, xcb::Rectangle::new(x, y, width, height));

        let height_diff = f64::from(global_config.height()) * (1.0 - value);
        let width_diff = f64::from(global_config.width()) * (1.0 - value);

        x += global_config.padding as i16;
        y += global_config.padding as i16;
        width -= global_config.padding as u16 * 2;
        height -= global_config.padding as u16 * 2;

        match global_config.fill_direction {
            config::Direction::Up => {
                y += height_diff as i16;
                height -= height_diff as u16;
            },
            config::Direction::Down => {
                height -= height_diff as u16;
            },
            config::Direction::Left => {
                x += width_diff as i16;
                width -= width_diff as u16;
            },
            config::Direction::Right => {
                width -= width_diff as u16;
            },
        }

        self.draw_rectangle(color_config.foreground, xcb::Rectangle::new(x, y, width, height));
    }

    pub fn show(
        &self,
        value: f64,
        global_config: &config::GlobalConfig,
        color_config: &config::ColorConfig,
    ) {
        xcb::map_window(&self.connection, self.window);
        self.configure_window(global_config);
        self.draw_bar(value, global_config, color_config);
        self.connection.flush();
    }

    pub fn hide(&self) {
        xcb::unmap_window(&self.connection, self.window);
        self.connection.flush();
    }
}
