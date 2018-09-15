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
            0, 0,
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

        let title = "x11-overlay-bar-rs";
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

    fn configure_window_attributes(&self, global_config: &config::GlobalConfig) {
        xcb::change_window_attributes(
            &self.connection,
            self.window,
            &[
                (xcb::CONFIG_WINDOW_WIDTH, global_config.width_to_margin()),
                (xcb::CONFIG_WINDOW_HEIGHT, global_config.width_to_margin()),
                (xcb::CONFIG_WINDOW_X, global_config.x()),
                (xcb::CONFIG_WINDOW_X, global_config.y()),
            ],
        );
    }

    fn draw_bar(&self, global_config: &config::GlobalConfig, color_config: &config::ColorConfig) {
        xcb::change_gc(&self.connection, self.gc, &[(xcb::GC_FOREGROUND, 0x00FF0000)]);
        // xcb::poly_fill_rectangle(&self.connection, self.window, self.gc, &[Rectangle::new()])
    }

    pub fn show(
        &self,
        value: f64,
        global_config: &config::GlobalConfig,
        color_config: &config::ColorConfig,
    ) {
        self.configure_window_attributes(global_config);
        xcb::map_window(&self.connection, self.window);
        self.connection.flush();
    }

    pub fn hide(&self) {
        xcb::unmap_window(&self.connection, self.window);
        self.connection.flush();
    }
}
