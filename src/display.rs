use xcb;

struct Display {
    connection: xcb::Connection,
    window_id: u32,
    screen_index: usize,
}

impl Display {
    fn init_window(&self) {
        let screen = self.connection
            .get_setup()
            .roots()
            .nth(self.screen_index)
            .expect("Expected screen to exist.");

        let values = [
            (xcb::CW_BACK_PIXEL, screen.white_pixel()),
            (xcb::CW_OVERRIDE_REDIRECT, 1),
        ];

        xcb::create_window(
            &self.connection,
            xcb::COPY_FROM_PARENT as u8,
            self.window_id,
            screen.root(),
            0, 0,
            150, 150,
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            screen.root_visual(),
            &values
        );
    }

    pub fn new() -> Result<Self, xcb::ConnError> {
        let (connection, screen_index) = xcb::Connection::connect(None)?;
        let screen_index = screen_index as usize;
        let window_id = connection.generate_id();

        let ret = Display {
            connection,
            window_id,
            screen_index,
        };

        ret.init_window();

        let title = "x11-overlay-bar-rs";
        xcb::change_property(
            &ret.connection,
            xcb::PROP_MODE_REPLACE as u8,
            ret.window_id,
            xcb::ATOM_WM_NAME,
            xcb::ATOM_STRING,
            8,
            title.as_bytes()
        );

        xcb::change_property(
            &ret.connection,
            xcb::PROP_MODE_REPLACE as u8,
            ret.window_id,
            xcb::ATOM_WM_CLASS,
            xcb::ATOM_STRING,
            8,
            title.as_bytes()
        );

        ret.connection.flush();

        Ok(ret)
    }

    pub fn show(&self) {
        xcb::map_window(&self.connection, self.window_id);
        self.connection.flush();
    }

    pub fn hide(&self) {
        xcb::unmap_window(&self.connection, self.window_id);
        self.connection.flush();
    }
}
