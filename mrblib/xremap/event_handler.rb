module Xremap
  class EventHandler
    # @param [Xremap::Config] config
    # @param [Xremap::Display] display
    def initialize(config, display)
      @display = display
      @active_window = ActiveWindow.new(display)
      @grab_manager  = GrabManager.new(config, display)
      @key_remap_compiler = KeyRemapCompiler.new(config, display)
      remap_keys
    end

    def handle_key_press(keycode, input_state)
      matched_mask, handler = @key_press_handlers[keycode].find { |handler_mask, _handler| bitmask_subset?(input_state, handler_mask) }
      if handler
        handler.call(input_state & ~matched_mask)
      else
        XlibWrapper.press_key(@display, XlibWrapper.keycode_to_keysym(@display, keycode), input_state)
      end
    end

    def handle_property_notify
      if @active_window.changed?
        remap_keys
      end
    end

    def handle_mapping_notify
      remap_keys
    end

    private

    def remap_keys
      window = @active_window.current_window
      @key_press_handlers = @key_remap_compiler.compile_for(window)
      @grab_manager.grab_keys_for(window)
    end

    def bitmask_subset?(parent, child)
      (parent & child) == child
    end
  end
end
