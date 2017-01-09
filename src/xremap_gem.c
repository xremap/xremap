#include "mruby.h"

extern void mrb_xremap_xlib_wrapper_init(mrb_state *mrb, struct RClass *mXremap);
extern void mrb_xremap_display_init(mrb_state *mrb, struct RClass *mXremap);
extern void mrb_xremap_x11_constants_init(mrb_state *mrb, struct RClass *mXremap);

void
mrb_xremap_gem_init(mrb_state *mrb)
{
  struct RClass *mXremap = mrb_define_module(mrb, "Xremap");

  mrb_xremap_xlib_wrapper_init(mrb, mXremap);
  mrb_xremap_display_init(mrb, mXremap);
  mrb_xremap_x11_constants_init(mrb, mXremap);
  mrb_gc_arena_restore(mrb, 0);
}

void
mrb_xremap_gem_final(mrb_state *mrb)
{
}
