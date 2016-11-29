#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <mruby.h>

mrb_value
load_config(mrb_state *mrb, char *filename)
{
  struct RClass *mXkremap = mrb_module_get(mrb, "Xkremap");
  struct RClass *cConfig  = mrb_class_get_under(mrb, mXkremap, "Config");

  mrb_value config = mrb_funcall(mrb, mrb_obj_value(cConfig), "load", 1, mrb_str_new_cstr(mrb, filename));
  if (mrb->exc) {
    mrb_print_error(mrb);
  }
  return config;
}
