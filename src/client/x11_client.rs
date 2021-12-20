pub struct X11Client {
    // Both of them are lazily initialized
    display: Option<*mut x11::xlib::Display>,
    supported: Option<bool>,
    last_wm_class: String,
}

impl X11Client {
    pub fn new() -> X11Client {
        X11Client {
            display: None,
            supported: None,
            last_wm_class: String::new(),
        }
    }

    pub fn supported(&mut self) -> bool {
        match self.supported {
            Some(supported) => supported,
            None => {
                let display = self.display();
                let mut focused_window = 0;
                let mut focus_state = 0;
                unsafe { x11::xlib::XGetInputFocus(display, &mut focused_window, &mut focus_state) };
                let supported = focused_window > 0;
                self.supported = Some(supported);
                supported
            },
        }
    }

    pub fn current_wm_class(&mut self) -> Option<String> {
        if !self.supported() {
            return None
        }

        let display = self.display();
        let mut focused_window = 0;
        let mut focus_state = 0;
        unsafe { x11::xlib::XGetInputFocus(display, &mut focused_window, &mut focus_state) };

        let mut x_class_hint = x11::xlib::XClassHint {
            res_name: std::ptr::null_mut(),
            res_class: std::ptr::null_mut(),
        };
        loop {
            unsafe {
                if x11::xlib::XGetClassHint(display, focused_window, &mut x_class_hint) == 1 {
                    break;
                }
            }

            let mut nchildren: u32 = 0;
            let mut root: x11::xlib::Window = 0;
            let mut parent: x11::xlib::Window = 0;
            let mut children: *mut x11::xlib::Window = &mut 0;
            unsafe {
                if x11::xlib::XQueryTree(
                    display,
                    focused_window,
                    &mut root,
                    &mut parent,
                    &mut children,
                    &mut nchildren,
                ) == 0
                {
                    break;
                }
            }
            if !children.is_null() {
                unsafe {
                    x11::xlib::XFree(children as *mut std::ffi::c_void);
                }
            }

            // The root client's parent is NULL. Avoid querying it to prevent SEGV on XGetClientHint.
            if parent == 0 {
                return None;
            }
            focused_window = parent;
        }

        if !x_class_hint.res_name.is_null() {
            unsafe {
                x11::xlib::XFree(x_class_hint.res_name as *mut std::ffi::c_void);
            }
        }

        if !x_class_hint.res_class.is_null() {
            let wm_class = unsafe {
                // Note: This seems to free `x_class_hint.res_class`. So XFree on it would double-free it.
                std::ffi::CString::from_raw(x_class_hint.res_class as *mut i8)
                    .into_string()
                    .unwrap()
            };
            if &self.last_wm_class != &wm_class {
                self.last_wm_class = wm_class.clone();
                println!("wm_class: {}", &wm_class);
            }
            Some(wm_class)
        } else {
            None
        }
    }

    fn display(&mut self) -> *mut x11::xlib::Display {
        match self.display {
            Some(display) => display,
            None => {
                let display = unsafe { x11::xlib::XOpenDisplay(std::ptr::null()) };
                self.display = Some(display);
                display
            }
        }
    }
}
