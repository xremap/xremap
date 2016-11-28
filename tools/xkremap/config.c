#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <mruby.h>

FILE*
open_file(char *filename)
{
  if (!strcmp(filename, "-"))
    return stdin;

  FILE *file = fopen(filename, "r");
  if (!file) {
    fprintf(stderr, "Failed to open: '%s'\n", filename);
    exit(1);
  }
  return file;
}

char*
read_file(char *filename)
{
  FILE *fp = open_file(filename);
  fseek(fp, 0L, SEEK_END);
  long size = ftell(fp);
  rewind(fp);

  char *ret = malloc(size * sizeof(char));
  fread(ret, sizeof(char), size, fp);
  fclose(fp);

  return ret;
}

mrb_value
load_config(mrb_state *mrb, char *filename)
{
  char *dsl = read_file(filename);

  struct RClass *mXkremap = mrb_module_get(mrb, "Xkremap");
  struct RClass *cConfig  = mrb_class_get_under(mrb, mXkremap, "Config");
  mrb_value config = mrb_funcall(mrb, mrb_obj_value(cConfig), "load", 1, mrb_str_new_cstr(mrb, dsl));
  if (mrb->exc) {
    mrb_print_error(mrb);
  }

  free(dsl);
  return config;
}
