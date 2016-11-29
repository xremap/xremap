module Xkremap
  class KeyRemapCompiler
    def initialize(config, display)
      @config  = config
      @display = display
      puts "Config loaded: #{@config.inspect}"
    end

    # @return [Hash] : keycode(Fixnum) -> state(Fixnum) -> handler(Proc)
    def compile
      result = Hash.new { |h, k| h[k] = {} }
      set_handlers(result)
      result
    end

    private

    def set_handlers(result)
      display = @display

      @config.remaps.each do |remap|
        from = remap.from_key
        tos  = remap.to_keys

        result[to_keycode(from.keysym)][from.modifier] = Proc.new do
          tos.each do |to|
            XlibWrapper.input_key(display, to.keysym, to.modifier)
          end
        end
      end
    end

    def to_keycode(keysym)
      XlibWrapper.keysym_to_keycode(@display, keysym)
    end
  end
end
