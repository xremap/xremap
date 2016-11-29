module Xkremap
  class EventHandler
    # @param [Xkremap::Config] config
    # @param [Xkremap::Display] display
    def initialize(config, display)
      @active_window = ActiveWindow.new(display)
      @grab_manager  = GrabManager.new(config, display)
      @key_remap_compiler = KeyRemapCompiler.new(config, display)
      remap_keys
    end

    def handle_key_press(keycode, state)
      handler = @key_press_handlers[keycode][state]
      if handler
        handler.call
      else
        $stderr.puts "Handler not found!: #{[keycode, state, @key_press_handlers].inspect}"
      end
    end

    def handle_property_notify
      if @active_window.changed?
        remap_keys
      end
    end

    def handle_mapping_notify
      puts 'mapping changed!'
      remap_keys
    end

    private

    def remap_keys
      window = @active_window.current_window
      @key_press_handlers = @key_remap_compiler.compile_for(window)
      @grab_manager.grab_keys_for(window)
      puts 'remap keys!'
    end
  end
end
