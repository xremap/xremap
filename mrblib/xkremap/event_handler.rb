module Xkremap
  class EventHandler
    # @param [Xkremap::Config] config
    # @param [Xkremap::Display] display
    def initialize(config, display)
      @config  = config
      @display = display
      XlibWrapper.grab_keys(@display)
    end

    def handle_key_press(keycode, state)
      puts 'Event: key_press'
    end

    def handle_property_notify
      puts 'Event: property_notify'
    end

    def handle_mapping_notify
      puts 'Event: mapping_notify'
    end
  end
end
