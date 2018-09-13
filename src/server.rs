use dbus::tree::{Interface, Factory, MTFn};
use dbus::{BusType, Connection, ConnectionItem, NameFlag, tree};
use display;
use std::sync::Arc;

fn create_interface(display: &Arc<display::Display>) -> Interface<MTFn<()>, ()> {
    let factory = Factory::new_fn::<()>();
    let show_display = display.clone();
    let hide_display = display.clone();
    factory.interface("com.test.bar", ())
        .add_m(
            factory.method("show", (), move |method_info| {
                show_display.show();

                Ok(vec!(method_info.msg.method_return()))
            })
                .inarg::<u8, _>("value")
                .inarg::<&str, _>("mode")
        )
        .add_m(factory.method("hide", (), move |method_info| {
            hide_display.hide();
            Ok(vec!(method_info.msg.method_return()))
        }))
        .add_m(factory.method("stop", (), move |method_info| {
            Ok(vec!(method_info.msg.method_return()))
        }))
}

fn create_tree(display: &Arc<display::Display>) -> tree::Tree<MTFn<()>, ()> {
    let factory = Factory::new_fn::<()>();
    let interface = create_interface(display);
    factory.tree(()).add(factory.object_path("/bar", ()).introspectable().add(interface))
}

pub fn start_server(display: display::Display) {
    let display = Arc::new(display);

    // Create connection and register bus name
    let connection = Connection::get_private(BusType::Session).unwrap();
    connection.register_name("com.test.bar", NameFlag::ReplaceExisting as u32).unwrap();

    let tree = create_tree(&display);

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
    }
}
