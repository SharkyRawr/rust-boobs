extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;

use gio::prelude::*;
use glib::prelude::*;
use gtk::prelude::*;

use gtk::{ApplicationWindow, MessageDialog};

use std::env;
use std::sync::{Arc, Mutex};

mod flickr;

fn get_pixmap_from_graphics_url(url: &str) -> Option<gdk_pixbuf::Pixbuf> {
    let bytes = flickr::download_url_as_bytes(url).unwrap();
    let glib_bytes = glib::Bytes::from(&bytes);
    let instream = gio::MemoryInputStream::new_from_bytes(&glib_bytes);
    let not_cancellable: Option<&gio::Cancellable> = None;
    match gdk_pixbuf::Pixbuf::new_from_stream(&instream, not_cancellable) {
                Ok(a) => Some(a),
                
                Err(e) => {
                    println!("Unable to get pixbuf from stream: {:?}", e);
                    None
                }
    }
}

fn set_image_pixmap_from_url_or_errmsg(
    main_window: &gtk::ApplicationWindow,
    image_widget: &gtk::Image,
    url: &str,
) {
    match get_pixmap_from_graphics_url(url) {
        Some(pb) => {
            image_widget.set_from_pixbuf(Some(&pb));
        }
        None => {
            print!("Unable to set pixbuf");
            let dlg = MessageDialog::new(
                Some(main_window),
                gtk::DialogFlags::empty(),
                gtk::MessageType::Error,
                gtk::ButtonsType::Ok,
                &"Could not download image!",
            );
            dlg.run();
            dlg.destroy();
        }
    };
}

fn reload_photos(arc_photos: &Arc<Mutex<Vec<String>>>, arc_main_window: &Arc<Mutex<gtk::ApplicationWindow>>) {
    let mut photos = arc_photos.lock().unwrap();
    *photos = match flickr::get_photos_by_tags("nsfw,boobs") {
        Ok(photos) => {
            println!("Found {} photos.", photos.len());
            photos
        },
        Err(e) => {
            print!("No photos found in response! {:?}", e);
            let dlg = MessageDialog::new(
                Some(&*(arc_main_window.lock().unwrap())),
                gtk::DialogFlags::empty(),
                gtk::MessageType::Error,
                gtk::ButtonsType::Ok,
                &"Could not find any photos in flickr feed?!",
            );
            dlg.run();
            dlg.destroy();
            Vec::new()
        }
    };
}

fn build_ui(app: &gtk::Application) {
    let main_window = ApplicationWindow::new(app);
    let arc_main_window = Arc::new(Mutex::new(main_window));

    arc_main_window.lock().unwrap().set_size_request(640, 480);
    arc_main_window
        .lock()
        .unwrap()
        .set_position(gtk::WindowPosition::Center);

    let hdr_bar = gtk::HeaderBar::new();
    hdr_bar.set_title(Some("Boobs"));
    hdr_bar.set_show_close_button(true);
    arc_main_window.lock().unwrap().set_titlebar(Some(&hdr_bar));

    let button_refresh = gtk::Button::new_from_icon_name(Some("view-refresh"), gtk::IconSize::Button);
    hdr_bar.add(&button_refresh);

    
    let arc_photos = Arc::new(Mutex::new(Vec::<String>::new()));
    let arc_photo_index = Arc::new(Mutex::new(0));

    reload_photos(&arc_photos, &arc_main_window);

    let image_container = gtk::Image::new();
    let image_event_box = gtk::EventBox::new();
    image_event_box.add(&image_container);
    //image_event_box.add_events(gtk::gdk::EventMask::BUTTON_PRESS_MASK | gtk::gdk::EventMask::BUTTON_RELEASE_MASK);
    arc_main_window.lock().unwrap().add(&image_event_box);

    set_image_pixmap_from_url_or_errmsg(
        &arc_main_window.lock().unwrap(),
        &image_container,
        &arc_photos.lock().unwrap()[*arc_photo_index.lock().unwrap() as usize],
    );
    *arc_photo_index.lock().unwrap() += 1;

    {
        let arc_main_window = arc_main_window.clone();
        let arc_photos = arc_photos.clone();

        image_event_box.connect_button_release_event(move |_, _| {
            
            set_image_pixmap_from_url_or_errmsg(
                &arc_main_window.lock().unwrap(),
                &image_container,
                &arc_photos.lock().unwrap()[*arc_photo_index.lock().unwrap() as usize],
            );

            let mut num = arc_photo_index.lock().unwrap();
            *num += 1;
            if *num >= arc_photos.lock().unwrap().len() {
                *num = 0;
            }
            println!("Index: {}", *num - 1);

            Inhibit(true)
        });
    }

    {
        let arc_main_window = arc_main_window.clone();
        let arc_photos = arc_photos.clone();
        button_refresh.connect_clicked(move |_| {
            reload_photos(&arc_photos, &arc_main_window);
        });
    }

    arc_main_window.lock().unwrap().show_all();
}

fn main() {
    let uiapp = gtk::Application::new(
        Some("pw.sharky.rust.gtk.boobs"),
        gio::ApplicationFlags::FLAGS_NONE,
    )
    .expect("Application::new failed");

    uiapp.connect_activate(|app| {
        build_ui(app);
    });
    uiapp.run(&env::args().collect::<Vec<_>>());

    //flickr::get_photos_by_tags("nsfw,boobs");
}
