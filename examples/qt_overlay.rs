use qt_widgets::QApplication;

fn main() {
    QApplication::init(|_| unsafe {
        let overlay = kanjisabi::qt::Overlay::new();
        overlay.show();
        QApplication::exec()
    })
}
