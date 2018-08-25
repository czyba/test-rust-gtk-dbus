use dbus::{Connection, BusType, NameFlag};
use dbus::tree::Factory;
use dbus::arg::RefArg;
use dbus::arg::Variant;

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use super::notification::Notification;


pub enum DBUSEvent {
    NotificationReceived(Notification),
}

use self::DBUSEvent::NotificationReceived;


pub struct DBusThread {
    // pub from_port_chan_rx: Receiver<SerialResponse>,
    pub from_dbus_chan_rx: Receiver<DBUSEvent>,
}

impl DBusThread {
    pub fn new<F: Fn() + Send + 'static>(callback: F) -> Self {

        let (from_dbus_chan_tx, from_dbus_chan_rx) = channel();
        // let (to_port_chan_tx, to_port_chan_rx) = channel();

        thread::spawn(move || {
            let c = Connection::get_private(BusType::Session).unwrap();
            c.register_name("org.freedesktop.Notifications", NameFlag::ReplaceExisting as u32).unwrap();

            let f = Factory::new_fn::<()>();

            // We create a tree with one object path inside and make that path introspectable.
            let tree = f.tree(()).add(f.object_path("/org/freedesktop/Notifications", ()).introspectable().add(

                // We add an interface to the object path...
                f.interface("org.freedesktop.Notifications", ())
                .add_m(
                    // ...and a method inside the interface.
                    f.method("GetCapabilities", (), move |m| {
                        let retval = vec!["body"];
                        let mret = m.msg.method_return().append1(retval);
                        Ok(vec!(mret))

                    // Our method has one output argument and one input argument.
                    }).outarg::<Vec<&str>,_>("reply"),
                // We also add the signal to the interface. This is mainly for introspection.
                ).add_m(f.method("Notify", (), move |m| {
                    let mut iter = m.msg.iter_init();
                    let app_name: &str  = try!(iter.read());
                    let replaces_id: u32 = try!(iter.read());
                    let _app_icon: &str = try!(iter.read());
                    let summary: &str = try!(iter.read());
                    let body: &str = try!(iter.read());

                    from_dbus_chan_tx.send(NotificationReceived(Notification {
                        app_name: String::from(app_name),
                        summary: String::from(summary),
                        body: String::from(body),
                        urgency: 1
                    })).unwrap();
                    callback();

                    let mret = m.msg.method_return().append1(5);
                    Ok(vec!(mret))
                }).inarg::<&str, _>("app_name")
                    .inarg::<u32, _>("replaces_id")
                    .inarg::<&str, _>("app_icon")
                    .inarg::<&str, _>("summary")
                    .inarg::<&str, _>("body")
                    .inarg::<Vec<&str>, _>("actions")
                    .inarg::<HashMap<&str, Variant<Box<RefArg>>>, _>("hints")
                    .inarg::<i32, _>("expire_timeout")
                    .outarg::<u32,_>("id")
                ).add_m(f.method("GetServerInformation", (), move |m| {
                        let mret = m.msg.method_return()
                            .append1("rnd")
                            .append1("cczyba.de")
                            .append1("0.1.0")
                            .append1("1.2");
                        Ok(vec!(mret))
                    }).outarg::<&str, _>("name")
                    .outarg::<&str, _>("vendor")
                    .outarg::<&str, _>("version")
                    .outarg::<&str, _>("spec_version")
                )
            ));

            // We register all object paths in the tree.
            tree.set_registered(&c, true).unwrap();

            // We add the tree to the connection so that incoming method calls will be handled
            // automatically during calls to "incoming".
            c.add_handler(tree);
            loop { c.incoming(1000).next(); }
        });

        DBusThread {
            from_dbus_chan_rx
        }

    }
}
