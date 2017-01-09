#include <X11/Xlib.h>
#include <mruby.h>

extern mrb_value mrb_wrap_x_display(mrb_state *mrb, Display *display);

mrb_value
new_event_handler(mrb_state *mrb, mrb_value config, Display *display)
{
  struct RClass *mXremap = mrb_module_get(mrb, "Xremap");
  struct RClass *cEventHandler = mrb_class_get_under(mrb, mXremap, "EventHandler");
  mrb_value display_obj = mrb_wrap_x_display(mrb, display);
  return mrb_funcall(mrb, mrb_obj_value(cEventHandler), "new", 2, config, display_obj);
}

void
handle_key_press(mrb_state *mrb, mrb_value event_handler, unsigned int keycode, unsigned int state)
{
  mrb_funcall(mrb, event_handler, "handle_key_press", 2,
      mrb_fixnum_value(keycode), mrb_fixnum_value(state));
  if (mrb->exc) {
    mrb_print_error(mrb);
  }
}

void
handle_property_notify(mrb_state *mrb, mrb_value event_handler)
{
  mrb_funcall(mrb, event_handler, "handle_property_notify", 0);
  if (mrb->exc) {
    mrb_print_error(mrb);
  }
}

void
handle_mapping_notify(mrb_state *mrb, mrb_value event_handler)
{
  mrb_funcall(mrb, event_handler, "handle_mapping_notify", 0);
  if (mrb->exc) {
    mrb_print_error(mrb);
  }
}
