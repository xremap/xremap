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

      # TODO: remove this log
      puts "remapped for class: #{XlibWrapper.fetch_window_class(@display, window).inspect}"
    end
  end
end
