extern crate dbus;
extern crate gtk;
extern crate gio;

use gtk::prelude::*;
use gio::prelude::*;

use std::env::args;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use dbus::{Connection, BusType, NameFlag};
use dbus::tree::Factory;
use dbus::arg::RefArg;
use dbus::arg::Variant;
use std::string::String;

mod notification;
use notification::Notification;

// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

fn build_ui(application: &gtk::Application, max_count: u32) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("First GTK+ Clock");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(260, 40);

    window.connect_delete_event(clone!(window => move |_, _| {
        window.destroy();
        Inhibit(false)
    }));

    window.show_all();

    // // we are using a closure to capture the label (else we could also use a normal function)
    // let tick = move || {
    //     let time = current_time();
    //     label.set_text(&time);
    //     // we could return gtk::Continue(false) to stop our clock after this tick
    //     gtk::Continue(true)
    // };

    // // executes the closure once every second
    // gtk::timeout_add_seconds(1, tick);
}

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));


    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Cairo API test");
    let label = Arc::new(Mutex::new(gtk::Label::new(None)));
    let label2 = label.clone();
    label.lock().unwrap().set_text(&1.to_string());
    window.add(&*label.lock().unwrap());
    window.show_all();

    let max_count: Arc<Mutex<u32>> = Arc::new(Mutex::new(1));
    let active_notifications: Arc<Mutex<HashMap<u32, Notification>>> = Arc::new(Mutex::new(HashMap::new()));
    let notify_max_count = max_count.clone();
    let callback = || {
        let max_id = max_count.lock().unwrap();
        label2.lock().unwrap().set_text(&max_id.to_string());
        gtk::Continue(false)
    };

    // Serve other peers forever.
    thread::spawn(move || {
        let c = Connection::get_private(BusType::Session).unwrap();
        c.register_name("org.freedesktop.Notifications", NameFlag::ReplaceExisting as u32).unwrap();

        let f = Factory::new_fn::<()>();

        let signal = Arc::new(f.signal("HelloHappened", ()));
        let signal2 = signal.clone();

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
                let mut noitification_id = replaces_id;

                {
                    let mut notification_map = active_notifications.lock().unwrap();
                    if replaces_id == 0 || notification_map.contains_key(&replaces_id) {
                        let mut max_id = notify_max_count.lock().unwrap();
                        noitification_id = *max_id;
                        *max_id += 1;
                    }
                    notification_map.insert(noitification_id, Notification {
                        app_name: String::from(app_name),
                        summary: String::from(summary),
                        body: String::from(body),
                        urgency: 1
                    });
                    let foo = notify_max_count.clone();
                    gtk::idle_add(callback);
                }

                println!("Received message:\napp: {}\nsummary: {}\nbody: {}\nid: {}", app_name, summary, body, replaces_id);
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
            ).add_s(signal2)
        ));

        // We register all object paths in the tree.
        tree.set_registered(&c, true).unwrap();

        // We add the tree to the connection so that incoming method calls will be handled
        // automatically during calls to "incoming".
        c.add_handler(tree);
        loop { c.incoming(1000).next(); }
    });

    gtk::main();
}
