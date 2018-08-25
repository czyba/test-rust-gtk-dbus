extern crate dbus;
extern crate gtk;
extern crate glib;

use gtk::prelude::*;

use std::cell::RefCell;

mod notification;
use notification::Notification;
mod rnd;
use rnd::DBusThread;
use rnd::DBUSEvent;

thread_local!(
    static GLOBAL: RefCell<Option<(gtk::Label, DBusThread)>> = RefCell::new(None)
);

fn receive_dbus_notification(label: &gtk::Label, notification: Notification) {
    label.set_text(
        &format!("Received message:\napp: {}\nsummary: {}\nbody: {}",
            notification.app_name,
            notification.summary,
            notification.body
    ));
}

fn receive() -> glib::Continue {
    GLOBAL.with(|global| {
        if let Some((ref mut label, ref dbus_channels)) = *global.borrow_mut() {
            match dbus_channels.from_dbus_chan_rx.try_recv() {
                Ok(DBUSEvent::NotificationReceived(notification)) => {
                    receive_dbus_notification(&label, notification)
                }
                Err(_) => {}
            }
        }
    });
    glib::Continue(false)
}

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));


    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Test");
    let label = gtk::Label::new(None);
    label.set_text("initial");
    window.add(&label);
    window.show_all();

    GLOBAL.with(move |global| {
        *global.borrow_mut() =
                Some((label.clone(), DBusThread::new(|| { glib::idle_add(receive); })));
    });

    gtk::main();
}
