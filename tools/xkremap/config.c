#include <stdio.h>
#include <mruby.h>

mrb_value
load_config(mrb_state *mrb, char *filename)
{
  printf("load: %s\n", filename);
  return mrb_nil_value();
}
