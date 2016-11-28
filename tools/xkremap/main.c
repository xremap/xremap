#include <stdlib.h>
#include <stdio.h>
#include <signal.h>
#include <mruby.h>
#include <X11/Xlib.h>
#include "xkremap.h"

int
error_handler(Display *display, XErrorEvent *event)
{
  // FIXME: log error properly
  fprintf(stderr, "error detected!\n");
  return 0;
}

void
event_loop(Display *display, mrb_state *mrb, mrb_value event_handler)
{
  XEvent event;
  while (1) {
    XNextEvent(display, &event);
    switch (event.type) {
      case KeyPress:
        handle_key_press(mrb, event_handler, event.xkey.serial, event.xkey.keycode, event.xkey.state);
        break;
      case KeyRelease:
        // ignore. Is it necessary to handle this?
        break;
      case PropertyNotify:
        handle_property_notify(mrb, event_handler, event.xproperty.serial);
        break;
      case MappingNotify:
        handle_mapping_notify(mrb, event_handler, event.xmapping.serial);
        break;
      default:
        fprintf(stderr, "unexpected event detected! (%d)\n", event.type);
        break;
    }
  }
}

int
main(int argc, char **argv)
{
  if (argc != 2) {
    fprintf(stderr, "Usage: xkremap <file>\n");
    return 1;
  }

  mrb_state *mrb   = mrb_open();
  mrb_value config = load_config(mrb, argv[1]);

  Display *display = XOpenDisplay(NULL);
  if (!display) {
    fprintf(stderr, "Failed to open connection with X server!\n");
    return 1;
  }

  XSetErrorHandler(error_handler);
  XSelectInput(display, XDefaultRootWindow(display), KeyPressMask | PropertyChangeMask);

  mrb_value event_handler = new_event_handler(mrb, config, display);
  event_loop(display, mrb, event_handler);

  XCloseDisplay(display);
  return 0;
}
