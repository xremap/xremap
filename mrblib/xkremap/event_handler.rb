module Xkremap
  class EventHandler
    # @param [Xkremap::Config] config
    # @param [Xkremap::Display] display
    def initialize(config, display)
      @key_remap_builder = KeyRemapBuilder.new(config, display)
      @key_grab_manager  = KeyGrabManager.new(display)
      remap_keys
    end

    def handle_key_press(keycode, state)
      handler = @key_remap_builder.prebuilt[keycode][state]
      handler && handler.call
      puts 'Event: key_press'
    end

    def handle_property_notify
      if @key_remap_builder.active_window_changed?
        remap_keys
      end
    end

    def handle_mapping_notify
      remap_keys
    end

    private

    def remap_keys
      @key_remap_builder.build
      @key_grab_manager.grab_keys
      puts 'remap keys!'
    end
  end
end
