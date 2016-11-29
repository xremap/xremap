module Xkremap
  class GrabManager
    # @param [Xkremap::Config] config
    # @param [Xkremap::Display] display
    def initialize(config, display)
      @config  = config
      @display = display
    end

    def grab_keys_for(window)
      XlibWrapper.ungrab_keys(@display)
      @config.remaps_for(@display, window).each do |remap|
        from = remap.from_key
        XlibWrapper.grab_key(@display, from.keysym, from.modifier)
      end
    end
  end
end
