module Xkremap
  class KeyGrabManager
    # @param [Xkremap::Display] display
    def initialize(display)
      @display = display
    end

    def grab_keys
      XlibWrapper.ungrab_keys(@display)
      XlibWrapper.grab_key(@display, 0x0062, 1<<2) # C-b
    end
  end
end
