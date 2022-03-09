use std::rc::Rc;

use qt_widgets::qt_core::qs;
use qt_widgets::qt_core::FocusPolicy;
use qt_widgets::qt_core::QBox;
use qt_widgets::qt_core::QFlags;
use qt_widgets::qt_core::WidgetAttribute;
use qt_widgets::qt_core::WindowType;
use qt_widgets::QLabel;
use qt_widgets::QWidget;

pub struct Overlay {
    overlay: QBox<QWidget>,
}

impl Overlay {
    pub fn new() -> Rc<Self> {
        unsafe {
            let overlay = QWidget::new_0a();

            let this = Rc::new(Overlay { overlay });
            this.init();
            this
        }
    }

    unsafe fn init(self: &Rc<Self>) {
        self.overlay.set_focus_policy(FocusPolicy::NoFocus);
        self.overlay
            .set_attribute_1a(WidgetAttribute::WAAlwaysStackOnTop);
        self.overlay
            .set_attribute_1a(WidgetAttribute::WAShowWithoutActivating);
        self.overlay
            .set_attribute_1a(WidgetAttribute::WATranslucentBackground);
        self.overlay
            .set_attribute_1a(WidgetAttribute::WATransparentForMouseEvents);
        self.overlay
            .set_attribute_1a(WidgetAttribute::WANoMousePropagation);
        self.overlay.set_window_flags(QFlags::from(
            WindowType::FramelessWindowHint
                | WindowType::X11BypassWindowManagerHint
                | WindowType::WindowStaysOnTopHint,
        ));

        self.overlay.set_style_sheet(&qs(
            ".QWidget{background-color:rgba(255,0,0,20);border: 1px solid red;}
            .QLabel{",
        ));
    }

    pub fn add_box(self: &Rc<Self>, x: i32, y: i32, w: i32, h: i32) {
        unsafe {
            let panel = QWidget::new_0a();
            panel.set_geometry_4a(x, y, w, h);
            panel.set_parent_1a(&self.overlay);
        }
    }

    pub fn add_text(self: &Rc<Self>, x: i32, y: i32, size: i32, text: &String) {
        unsafe {
            let formatted = std::format!("<p style=\"font-size:{}px\">{}</p>", size, text);
            let label = QLabel::from_q_string(&qs(formatted.as_str()));
            label.set_geometry_4a(x, y, label.width(), label.height());
            label.set_parent_1a(&self.overlay);
        }
    }

    pub fn show(self: &Rc<Self>) {
        unsafe {
            self.overlay.show_full_screen();
        }
    }
}
