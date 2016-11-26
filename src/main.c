#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <X11/Xlib.h>
#include <X11/keysym.h>

int
error_handler(Display *display, XErrorEvent *event)
{
  // FIXME: log error properly
  fprintf(stderr, "error detected!\n");
  return 0;
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
input_key(Display *display, KeySym keysym, unsigned int modifiers)
{
  Window focus_win;
  int focus_state;
  XGetInputFocus(display, &focus_win, &focus_state);

  XKeyEvent event = create_key_event(display, focus_win, keysym, modifiers, KeyPress);
  XSendEvent(display, focus_win, True, KeyPressMask, (XEvent *)&event);

  event = create_key_event(display, focus_win, keysym, modifiers, KeyRelease);
  XSendEvent(display, focus_win, True, KeyReleaseMask, (XEvent *)&event);
}

void
grab_keys(Display *display)
{
  Window root = XDefaultRootWindow(display);
  XGrabKey(display, XKeysymToKeycode(display, XK_b), ControlMask, root, True, GrabModeAsync, GrabModeAsync);
}

void
event_loop(Display *display)
{
  XEvent event;
  while (1) {
    XNextEvent(display, &event);
    switch (event.type) {
      case KeyPress:
        input_key(display, XK_Left, 0);
        break;
      case KeyRelease:
        // ignore. Is it necessary to handle this?
        break;
      case MappingNotify:
        // FIXME: refresh mapping and grag keys again
        fprintf(stderr, "mapping notify detected!\n");
        break;
      default:
        fprintf(stderr, "unexpected event detected! (%d)\n", event.type);
        break;
    }
  }
}

void
remap_keys()
{
  Display *display = XOpenDisplay(NULL);
  if (!display) {
    fprintf(stderr, "Failed to open connection with X server!\n");
    exit(1);
  }

  XSetErrorHandler(error_handler);
  XSelectInput(display, XDefaultRootWindow(display), KeyPressMask);
  grab_keys(display);

  event_loop(display);

  // Close this on atexit?
  XCloseDisplay(display);
}

int
main(int argc, char **argv)
{
  remap_keys();
  return 0;
}
