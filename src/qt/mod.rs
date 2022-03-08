use std::rc::Rc;

use qt_widgets::qt_core::qs;
use qt_widgets::qt_core::FocusPolicy;
use qt_widgets::qt_core::QBox;
use qt_widgets::qt_core::QFlags;
use qt_widgets::qt_core::WidgetAttribute;
use qt_widgets::qt_core::WindowType;
use qt_widgets::QPushButton;
use qt_widgets::QWidget;

pub struct Overlay {
    overlay: QBox<QWidget>,
    panel: QBox<QWidget>,
    button: QBox<QPushButton>,
}

impl Overlay {
    pub fn new() -> Rc<Self> {
        unsafe {
            let overlay = QWidget::new_0a();

            let panel = QWidget::new_0a();
            panel.set_parent_1a(&overlay);

            let button = QPushButton::from_q_string(&qs("^^"));
            button.set_parent_1a(&panel);

            let this = Rc::new(Overlay {
                overlay,
                panel,
                button,
            });
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

        // self.widget.set_style_sheet(&qs("color:#FFFFFFFF"));

        self.panel.set_fixed_size_1a(&self.overlay.frame_size());
        self.overlay.set_style_sheet(&qs(
            ".QWidget{background-color:rgba(255,0,0,20);border: 4px solid red;padding: 6px;}",
        ));

        self.panel.set_geometry_4a(50, 50, 400, 200);
        self.button.set_geometry_4a(200, 130, 100, 40);
    }

    pub fn show(self: &Rc<Self>) {
        unsafe {
            self.overlay.show_full_screen();
        }
    }
}