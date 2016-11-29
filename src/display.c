#include <X11/Xlib.h>
#include "mruby.h"
#include "mruby/data.h"

struct mrb_display {
  Display *display;
};

static void
mrb_display_free(mrb_state *mrb, void *ptr)
{
  struct mrb_display *display = (struct mrb_display *)ptr;
  if (display != NULL) {
    mrb_free(mrb, display);
  }
}

struct mrb_data_type mrb_display_type = { "Display", mrb_display_free };

mrb_value
mrb_wrap_x_display(mrb_state *mrb, Display *display)
{
  struct RClass *mXkremap = mrb_module_get(mrb, "Xkremap");
  struct RClass *cDisplay = mrb_class_get_under(mrb, mXkremap, "Display");

  struct mrb_display *display_ptr = (struct mrb_display *)mrb_malloc(mrb, sizeof(struct mrb_display));
  display_ptr->display = display;
  mrb_value display_obj = mrb_obj_value(mrb_data_object_alloc(mrb, cDisplay, NULL, &mrb_display_type));
  DATA_TYPE(display_obj) = &mrb_display_type;
  DATA_PTR(display_obj)  = display_ptr;
  return display_obj;
}

Display*
extract_x_display(mrb_state *mrb, mrb_value display_obj)
{
  struct mrb_display *display_ptr = (struct mrb_display *)mrb_get_datatype(mrb, display_obj, &mrb_display_type);
  return display_ptr->display;
}

void
mrb_xkremap_display_init(mrb_state *mrb, struct RClass *mXkremap)
{
  mrb_define_class_under(mrb, mXkremap, "Display", mrb->object_class);
}
