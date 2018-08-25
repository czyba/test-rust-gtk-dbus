extern crate dbus;
extern crate gtk;
extern crate gio;
extern crate glib;

use gtk::prelude::*;
use gio::prelude::*;

use std::env::args;
use std::sync::Arc;
use std::sync::Mutex;
use std::string::String;
use std::cell::RefCell;

mod notification;
use notification::Notification;
mod rnd;
use rnd::DBusThread;

// make moving clones into closures more convenient
// macro_rules! clone {
//     (@param _) => ( _ );
//     (@param $x:ident) => ( $x );
//     ($($n:ident),+ => move || $body:expr) => (
//         {
//             $( let $n = $n.clone(); )+
//             move || $body
//         }
//     );
//     ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
//         {
//             $( let $n = $n.clone(); )+
//             move |$(clone!(@param $p),)+| $body
//         }
//     );
// }

// fn build_ui(application: &gtk::Application, max_count: u32) {
//     let window = gtk::ApplicationWindow::new(application);

//     window.set_title("First GTK+ Clock");
//     window.set_border_width(10);
//     window.set_position(gtk::WindowPosition::Center);
//     window.set_default_size(260, 40);

//     window.connect_delete_event(clone!(window => move |_, _| {
//         window.destroy();
//         Inhibit(false)
//     }));

//     window.show_all();

//     // // we are using a closure to capture the label (else we could also use a normal function)
//     // let tick = move || {
//     //     let time = current_time();
//     //     label.set_text(&time);
//     //     // we could return gtk::Continue(false) to stop our clock after this tick
//     //     gtk::Continue(true)
//     // };

//     // // executes the closure once every second
//     // gtk::timeout_add_seconds(1, tick);
// }

// fn receive() -> gtk::Continue {

// }

thread_local!(
    static GLOBAL: RefCell<Option<(gtk::Label, DBusThread)>> = RefCell::new(None)
);

fn receive() -> glib::Continue {
    GLOBAL.with(|global| {
        if let Some((ref mut label, ref dbus_channels)) = *global.borrow_mut() {
            label.set_text("foobar");
        }
    });

    // label.set_text("bar");
    glib::Continue(false)
}

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));


    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Test");
    let label = gtk::Label::new(None);
    // let label2 = label.clone();
    // label.lock().unwrap().set_text(&1.to_string());
    label.set_text("initial");
    window.add(&label);
    window.show_all();

    // let max_count: Arc<Mutex<u32>> = Arc::new(Mutex::new(1));
    // let active_notifications: Arc<Mutex<HashMap<u32, Notification>>> = Arc::new(Mutex::new(HashMap::new()));
    // let notify_max_count = max_count.clone();
    // let callback = || {
    //     let max_id = max_count.lock().unwrap();
    //     label2.lock().unwrap().set_text(&max_id.to_string());
    //     gtk::Continue(false)
    // };

    GLOBAL.with(move |global| {
        *global.borrow_mut() =
                Some((label.clone(), DBusThread::new(|| { glib::idle_add(receive); })));
    });


    gtk::main();
}
