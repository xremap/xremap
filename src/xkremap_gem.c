#include "mruby.h"

extern void mrb_mruby_xkremap_gem_init(mrb_state *mrb);
extern void mrb_xkremap_xlib_wrapper_init(mrb_state *mrb, struct RClass *mXkremap);
extern void mrb_xkremap_display_init(mrb_state *mrb, struct RClass *mXkremap);

void
mrb_xkremap_gem_init(mrb_state *mrb)
{
  struct RClass *mXkremap = mrb_define_module(mrb, "Xkremap");

  mrb_xkremap_xlib_wrapper_init(mrb, mXkremap);
  mrb_xkremap_display_init(mrb, mXkremap);
  mrb_gc_arena_restore(mrb, 0);
}

void
mrb_xkremap_gem_final(mrb_state *mrb)
{
}
