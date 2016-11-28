module Xkremap
  class EventHandler
    # @param [Xkremap::Config] config
    # @param [Xkremap::Display] display
    def initialize(config, display)
      @config  = config
      @display = display
    end

    def handle_key_press(serial, keycode, state)
      p serial
    end

    def handle_property_notify(serial)
    end

    def handle_mapping_notify(serial)
    end
  end
end
