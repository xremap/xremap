#include <X11/keysym.h>
#include <X11/XF86keysym.h>
#include <X11/X.h>
#include "mruby.h"

void
mrb_xremap_x11_constants_init(mrb_state *mrb, struct RClass *mXremap)
{
  struct RClass *mX11 = mrb_define_module_under(mrb, mXremap, "X11");
# define define_x11_const(name) mrb_define_const(mrb, mX11, #name, mrb_fixnum_value(name))

  // original constant.
  mrb_define_const(mrb, mX11, "NoModifier", mrb_fixnum_value(0));
#include "x11_constants_keysymdef.inc"
#include "x11_constants_X.inc"
#include "x11_constants_XF86keysym.inc"
}
