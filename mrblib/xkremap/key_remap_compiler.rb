module Xkremap
  class KeyRemapCompiler
    def initialize(config, display)
      @config  = config
      @display = display
      puts "Config loaded: #{@config.inspect}"
    end

    # @return [Hash] : keycode(Fixnum) -> state(Fixnum) -> handler(Proc)
    def compile_for(window)
      result = Hash.new { |h, k| h[k] = {} }
      set_handlers_for(result, window)
      result
    end

    private

    def set_handlers_for(result, window)
      @config.remaps_for(@display, window).each do |remap|
        from = remap.from_key
        tos  = remap.to_keys

        result[to_keycode(from.keysym)][from.modifier] = Proc.new do
          tos.each do |to|
            XlibWrapper.input_key(@display, to.keysym, to.modifier)
          end
        end
      end
    end

    def to_keycode(keysym)
      XlibWrapper.keysym_to_keycode(@display, keysym)
    end
  end
end
