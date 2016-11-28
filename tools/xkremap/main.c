#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <X11/Xatom.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
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

Window
get_focused_window(Display *display)
{
  Window window;
  int focus_state;

  XGetInputFocus(display, &window, &focus_state);
  return window;
}

void
input_key(Display *display, KeySym keysym, unsigned int modifiers)
{
  Window window = get_focused_window(display);

  XKeyEvent event = create_key_event(display, window, keysym, modifiers, KeyPress);
  XSendEvent(display, window, True, KeyPressMask, (XEvent *)&event);

  event = create_key_event(display, window, keysym, modifiers, KeyRelease);
  XSendEvent(display, window, True, KeyReleaseMask, (XEvent *)&event);
}

void
grab_keys(Display *display)
{
  Window root = XDefaultRootWindow(display);
  XGrabKey(display, XKeysymToKeycode(display, XK_b), ControlMask, root, True, GrabModeAsync, GrabModeAsync);
}

void
fetch_window_name(Display *display, Window window)
{
  Atom net_wm_name = XInternAtom(display, "_NET_WM_NAME", True);
  XTextProperty prop;

  XGetTextProperty(display, window, &prop, net_wm_name);
  if (prop.nitems == 0) {
    XGetWMName(display, window, &prop);
  }

  if (prop.nitems > 0 && prop.value) {
    if (prop.encoding == XA_STRING) {
      printf("XA_STRING: %s\n", (char *)prop.value);
    } else {
      char **l = NULL;
      int count;
      XmbTextPropertyToTextList(display, &prop, &l, &count);
      if (count > 0 && *l)
        printf("Not XA_STRING: %s\n", *l);
      XFreeStringList(l);
    }
  }
}

void
fetch_window_class(Display *display, Window window)
{
  Atom net_wm_name = XInternAtom(display, "WM_CLASS", True);
  XTextProperty prop;

  XGetTextProperty(display, window, &prop, net_wm_name);
  if (prop.nitems > 0 && prop.value) {
    if (prop.encoding == XA_STRING) {
      printf("class XA_STRING: %s\n", (char *)prop.value);
    } else {
      char **l = NULL;
      int count;
      XmbTextPropertyToTextList(display, &prop, &l, &count);
      if (count > 0 && *l)
        printf("class Not XA_STRING: %s\n", *l);
      XFreeStringList(l);
    }
  }
}

void
fetch_window_pid(Display *display, Window window)
{
  int format;
  unsigned long nitems, after;
  unsigned char *data = NULL;
  Atom ret_type;
  Atom targ_atom = XInternAtom(display, "_NET_WM_PID", True);
  if (XGetWindowProperty(display, window, targ_atom, 0, 65536, False,
        XA_CARDINAL, &ret_type, &format, &nitems, &after, &data) == Success && nitems > 0) {
    unsigned char *r = data;
    if (r) {
      int pid = (r[1] * 256) + r[0];
      printf("pid: %d\n", pid);
      XFree(r);
    }
  }
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
      case PropertyNotify:
        fetch_window_name(display, get_focused_window(display));
        fetch_window_class(display, get_focused_window(display));
        fetch_window_pid(display, get_focused_window(display));
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
  XSelectInput(display, XDefaultRootWindow(display), KeyPressMask | PropertyChangeMask);
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
