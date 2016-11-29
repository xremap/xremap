module Xkremap
  class EventHandler
    # @param [Xkremap::Config] config
    # @param [Xkremap::Display] display
    def initialize(config, display)
      @active_window  = ActiveWindow.new(display)
      @grab_manager   = GrabManager.new(display)
      @remap_compiler = KeyRemapCompiler.new(config, display)
      remap_keys
    end

    def handle_key_press(keycode, state)
      handler = @remap_handlers[keycode][state]
      if handler
        handler.call
      else
        $stderr.puts "Handler not found!: #{[keycode, state, @remap_handlers].inspect}"
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
      @remap_handlers = @remap_compiler.compile
      @grab_manager.grab_keys
      puts 'remap keys!'
    end
  end
end
