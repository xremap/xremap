#include <stdlib.h>
#include <stdio.h>
#include <signal.h>
#include <inttypes.h>
#include <mruby.h>
#include <X11/Xlib.h>
#include "xremap.h"

void
print_client_message_event(XClientMessageEvent *event)
{
  fprintf(stderr,
          "received ClientMesssage(message_type=%" PRIu32 " format=%d data=%#lx, %#lx, %#lx, %#lx, %#lx)",
          (uint32_t)event->message_type,
          event->format,
          (unsigned long)event->data.l[0],
          (unsigned long)event->data.l[1],
          (unsigned long)event->data.l[2],
          (unsigned long)event->data.l[3],
          (unsigned long)event->data.l[4]);

}

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
        handle_key_press(mrb, event_handler, event.xkey.keycode, event.xkey.state);
        break;
      case KeyRelease:
        // ignore. Is it necessary to handle this?
        break;
      case PropertyNotify:
        handle_property_notify(mrb, event_handler);
        break;
      case MappingNotify:
        handle_mapping_notify(mrb, event_handler);
        break;
      case ClientMessage:
        print_client_message_event((XClientMessageEvent*)&event);
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
    fprintf(stderr, "Usage: xremap <file>\n");
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
