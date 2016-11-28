#include <X11/Xlib.h>
#include <mruby.h>

mrb_value
new_event_handler(mrb_state *mrb, mrb_value config, Display *display)
{
  return mrb_nil_value();
}

void
handle_key_press(mrb_state *mrb, mrb_value event_handler, unsigned long serial, unsigned int keycode, unsigned int state)
{
}

void
handle_property_notify(mrb_state *mrb, mrb_value event_handler, unsigned long serial)
{
}

void
handle_mapping_notify(mrb_state *mrb, mrb_value event_handler, unsigned long serial)
{
}
