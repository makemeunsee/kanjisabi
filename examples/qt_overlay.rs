use qt_widgets::QApplication;

fn main() {
    QApplication::init(|_| unsafe {
        let overlay = kanjisabi::qt::Overlay::new();
        overlay.add_box(700, 800, 150, 20);
        overlay.add_box(1000, 500, 300, 300);
        overlay.add_text(1050, 500, 48, &String::from("yooooo"));

        overlay.show();

        QApplication::exec()
    })
}
