use config;
use dbus::tree::{Interface, Factory, MTFn};
use dbus::{BusType, Connection, ConnectionItem, NameFlag, tree};
use display;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

const BUS_NAME: &'static str = "com.jeffreyxiao.rob";
const INTERFACE_NAME: &'static str = "com.jeffreyxiao.rob";

fn create_interface(
    display: &Rc<display::Display>,
    global_config: config::GlobalConfig,
    color_configs: HashMap<String, config::ColorConfig>,
    is_running: Rc<Cell<bool>>,
) -> Interface<MTFn<()>, ()> {
    let factory = Factory::new_fn::<()>();
    let show_display = display.clone();
    let hide_display = display.clone();
    factory.interface(INTERFACE_NAME, ())
        .add_m(
            factory.method("show", (), move |method_info| {
                let (value, profile): (Option<f64>, Option<String>) = method_info.msg.get2();
                let value = value.ok_or(tree::MethodErr::invalid_arg(&"Expected [double, string]"))?;
                let profile = profile.ok_or(tree::MethodErr::invalid_arg(&"Expected [double, string]"))?;

                match color_configs.get(&profile) {
                    Some(ref color_config) => {
                        show_display.show(value, &global_config, color_config);
                    },
                    None => panic!(format!("Did not find color profile `{}`", profile)),
                };

                Ok(vec!(method_info.msg.method_return()))
            })
                .inarg::<f64, _>("profile")
                .inarg::<&str, _>("value")
        )
        .add_m(factory.method("hide", (), move |method_info| {
            hide_display.hide();
            Ok(vec!(method_info.msg.method_return()))
        }))
        .add_m(factory.method("stop", (), move |method_info| {
            is_running.set(false);
            println!("is running is {}", is_running.get());
            Ok(vec!(method_info.msg.method_return()))
        }))
}

fn create_tree(
    display: &Rc<display::Display>,
    global_config: config::GlobalConfig,
    color_configs: HashMap<String, config::ColorConfig>,
    is_running: Rc<Cell<bool>>,
) -> tree::Tree<MTFn<()>, ()> {
    let factory = Factory::new_fn::<()>();
    let interface = create_interface(display, global_config, color_configs, is_running);
    factory.tree(()).add(factory.object_path("/rob", ()).introspectable().add(interface))
}

pub fn start_server(
    display: display::Display,
    global_config: config::GlobalConfig,
    color_configs: HashMap<String, config::ColorConfig>,
) {
    let is_running = Rc::new(Cell::new(true));
    let display = Rc::new(display);

    // Create connection and register bus name
    let connection = Connection::get_private(BusType::Session).unwrap();
    connection.register_name(BUS_NAME, NameFlag::ReplaceExisting as u32).unwrap();

    let tree = create_tree(&display, global_config, color_configs, is_running.clone());

    // Register object paths
    tree.set_registered(&connection, true).unwrap();

    connection.add_handler(tree);

    for connection_item in connection.iter(1000) {
        match connection_item {
            ConnectionItem::Signal(_) => println!("Received signal"),
            ConnectionItem::MethodReturn(_) => println!("Received method"),
            ConnectionItem::Nothing => display.hide(),
            _ => {
                println!("here");
            },
        }

        if !is_running.get() {
            break;
        }
    }
}
