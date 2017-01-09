#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <mruby.h>

mrb_value
load_config(mrb_state *mrb, char *filename)
{
  struct RClass *mXremap = mrb_module_get(mrb, "Xremap");
  struct RClass *cConfig  = mrb_class_get_under(mrb, mXremap, "Config");

  mrb_value config = mrb_funcall(mrb, mrb_obj_value(cConfig), "load", 1, mrb_str_new_cstr(mrb, filename));
  if (mrb->exc) {
    mrb_print_error(mrb);
  }
  return config;
}
