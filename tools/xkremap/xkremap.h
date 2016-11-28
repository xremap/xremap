#include <mruby.h>
#include <X11/Xlib.h>

// config.c
mrb_value load_config(mrb_state *mrb, char *filename);

// event_handler.c
mrb_value new_event_handler(mrb_state *mrb, mrb_value config, Display *display);
void handle_key_press(mrb_state *mrb, mrb_value event_handler, unsigned int state, unsigned int keycode);
void handle_property_notify(mrb_state *mrb, mrb_value event_handler);
void handle_mapping_notify(mrb_state *mrb, mrb_value event_handler);
