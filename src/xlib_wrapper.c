#include <X11/Xlib.h>
#include <X11/keysym.h>
#include "mruby.h"

extern Display* extract_x_display(mrb_state *mrb, mrb_value display_obj);

mrb_value
mrb_xw_grab_keys(mrb_state *mrb, mrb_value self)
{
  mrb_value display_obj;
  mrb_get_args(mrb, "o", &display_obj);
  Display *display = extract_x_display(mrb, display_obj);

  Window root = XDefaultRootWindow(display);
  XGrabKey(display, XKeysymToKeycode(display, XK_b), ControlMask, root, True, GrabModeAsync, GrabModeAsync);

  return mrb_nil_value();
}

void
mrb_xkremap_xlib_wrapper_init(mrb_state *mrb, struct RClass *mXkremap)
{
  struct RClass *cXlibWrapper = mrb_define_class_under(mrb, mXkremap, "XlibWrapper", mrb->object_class);
  mrb_define_class_method(mrb, cXlibWrapper, "grab_keys", mrb_xw_grab_keys, MRB_ARGS_REQ(1));
}
