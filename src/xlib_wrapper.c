#include <X11/Xatom.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/keysym.h>
#include "mruby.h"

extern Display* extract_x_display(mrb_state *mrb, mrb_value display_obj);

mrb_value
mrb_xw_fetch_window_class(mrb_state *mrb, mrb_value self)
{
  mrb_value display_obj;
  mrb_int window;
  mrb_get_args(mrb, "oi", &display_obj, &window);

  Display *display = extract_x_display(mrb, display_obj);
  Atom net_wm_name = XInternAtom(display, "WM_CLASS", True);

  XTextProperty prop;
  XGetTextProperty(display, window, &prop, net_wm_name);

  mrb_value ret;
  if (prop.nitems > 0 && prop.value) {
    if (prop.encoding == XA_STRING) {
      ret = mrb_str_new_cstr(mrb, (char *)prop.value);
    } else {
      char **l = NULL;
      int count;
      XmbTextPropertyToTextList(display, &prop, &l, &count);
      if (count > 0 && *l) {
        ret = mrb_str_new_cstr(mrb, *l);
      } else {
        ret = mrb_str_new_cstr(mrb, "");
      }
      XFreeStringList(l);
    }
  }
  return ret;
}

Window
get_focused_window(Display *display)
{
  Window window;
  int focus_state;

  XGetInputFocus(display, &window, &focus_state);
  return window;
}

XKeyEvent
create_key_event(Display *display, Window window, KeySym keysym, unsigned int modifiers, int type)
{
  return (XKeyEvent){
    .display     = display,
    .window      = window,
    .root        = XDefaultRootWindow(display),
    .subwindow   = None,
    .time        = CurrentTime,
    .x           = 1,
    .y           = 1,
    .x_root      = 1,
    .y_root      = 1,
    .same_screen = True,
    .keycode     = XKeysymToKeycode(display, keysym),
    .state       = modifiers,
    .type        = type,
  };
}

void
send_press_event(Display *display, Window window, KeySym keysym, unsigned int modifiers)
{
  XKeyEvent event = create_key_event(display, window, keysym, modifiers, KeyPress);
  XSendEvent(display, window, True, KeyPressMask, (XEvent *)&event);
}

void
send_release_event(Display *display, Window window, KeySym keysym, unsigned int modifiers)
{
  XKeyEvent event = create_key_event(display, window, keysym, modifiers, KeyRelease);
  XSendEvent(display, window, True, KeyReleaseMask, (XEvent *)&event);
}

mrb_value
mrb_xw_press_key(mrb_state *mrb, mrb_value self)
{
  mrb_value display_obj;
  mrb_int keysym, modifiers;
  mrb_get_args(mrb, "oii", &display_obj, &keysym, &modifiers);

  Display *display = extract_x_display(mrb, display_obj);
  Window window = get_focused_window(display);

  send_press_event(display, window, keysym, modifiers);

  return mrb_nil_value();
}

mrb_value
mrb_xw_release_key(mrb_state *mrb, mrb_value self)
{
  mrb_value display_obj;
  mrb_int keysym, modifiers;
  mrb_get_args(mrb, "oii", &display_obj, &keysym, &modifiers);

  Display *display = extract_x_display(mrb, display_obj);
  Window window = get_focused_window(display);

  send_release_event(display, window, keysym, modifiers);

  return mrb_nil_value();
}

mrb_value
mrb_xw_input_key(mrb_state *mrb, mrb_value self)
{
  mrb_value display_obj;
  mrb_int keysym, modifiers;
  mrb_get_args(mrb, "oii", &display_obj, &keysym, &modifiers);

  Display *display = extract_x_display(mrb, display_obj);
  Window window = get_focused_window(display);

  send_press_event(display, window, keysym, modifiers);
  send_release_event(display, window, keysym, modifiers);

  return mrb_nil_value();
}

mrb_value
mrb_xw_keysym_to_keycode(mrb_state *mrb, mrb_value self)
{
  mrb_value display_obj;
  mrb_int keysym;
  mrb_get_args(mrb, "oi", &display_obj, &keysym);

  Display *display = extract_x_display(mrb, display_obj);
  return mrb_fixnum_value(XKeysymToKeycode(display, keysym));
}

mrb_value
mrb_xw_fetch_active_window(mrb_state *mrb, mrb_value self)
{
  mrb_value display_obj;
  mrb_int keycode, state;
  mrb_get_args(mrb, "o", &display_obj, &keycode, &state);

  Display *display = extract_x_display(mrb, display_obj);
  return mrb_fixnum_value(get_focused_window(display));
}

mrb_value
mrb_xw_grab_key(mrb_state *mrb, mrb_value self)
{
  mrb_value display_obj;
  mrb_int keycode, state;
  mrb_get_args(mrb, "oii", &display_obj, &keycode, &state);

  Display *display = extract_x_display(mrb, display_obj);
  XGrabKey(display, XKeysymToKeycode(display, keycode), state, XDefaultRootWindow(display), True, GrabModeAsync, GrabModeAsync);

  return mrb_nil_value();
}

mrb_value
mrb_xw_ungrab_keys(mrb_state *mrb, mrb_value self)
{
  mrb_value display_obj;
  mrb_get_args(mrb, "o", &display_obj);

  Display *display = extract_x_display(mrb, display_obj);
  XUngrabKey(display, AnyKey, AnyModifier, XDefaultRootWindow(display));

  return mrb_nil_value();
}

void
mrb_xkremap_xlib_wrapper_init(mrb_state *mrb, struct RClass *mXkremap)
{
  struct RClass *cXlibWrapper = mrb_define_class_under(mrb, mXkremap, "XlibWrapper", mrb->object_class);
  mrb_define_class_method(mrb, cXlibWrapper, "input_key",           mrb_xw_input_key,           MRB_ARGS_REQ(3));
  mrb_define_class_method(mrb, cXlibWrapper, "press_key",           mrb_xw_press_key,           MRB_ARGS_REQ(3));
  mrb_define_class_method(mrb, cXlibWrapper, "release_key",         mrb_xw_release_key,         MRB_ARGS_REQ(3));
  mrb_define_class_method(mrb, cXlibWrapper, "keysym_to_keycode",   mrb_xw_keysym_to_keycode,   MRB_ARGS_REQ(2));
  mrb_define_class_method(mrb, cXlibWrapper, "fetch_active_window", mrb_xw_fetch_active_window, MRB_ARGS_REQ(1));
  mrb_define_class_method(mrb, cXlibWrapper, "fetch_window_class",  mrb_xw_fetch_window_class,  MRB_ARGS_REQ(2));
  mrb_define_class_method(mrb, cXlibWrapper, "grab_key",            mrb_xw_grab_key,            MRB_ARGS_REQ(3));
  mrb_define_class_method(mrb, cXlibWrapper, "ungrab_keys",         mrb_xw_ungrab_keys,         MRB_ARGS_REQ(1));
}
