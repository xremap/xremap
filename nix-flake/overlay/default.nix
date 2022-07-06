# Overlay file that contains the definition of building a package
xremap: naersk-lib: pkgs: { withSway ? false, withGnome ? false, withX11 ? false }:
{
  xremap-unwrapped = naersk-lib.buildPackage rec {
    # Used in this context, "xremap" will translate to the path in the store where the source was downloaded
    root = xremap;
    cargoBuildOptions = with pkgs.lib;
      (x:
        x ++ optional (builtins.any (x: x == true) [ withSway withGnome ]) "--features \"${ if withSway then "sway" else ""} ${if withGnome then "gnome" else ""} ${ if withX11 then "x11" else ""}\""
      );
    # The following two options are for introspection to be able to see if sway/gnome were actually pulled in
    # To see that - visually inspect the deps directory inside result/target/ and check for swayipc/zbus
    # See cargo.toml for feature-specific deps
    /* copyTarget = true; */
    /* compressTarget = false; */
  };
}
