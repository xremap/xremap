module Xkremap
  class KeyRemapCompiler
    def initialize(config, display)
      @config  = config
      @display = display
    end

    # @return [Hash] : keycode(Fixnum) -> state(Fixnum) -> handler(Proc)
    def compile
      result = Hash.new { |h, k| h[k] = {} }
      result[to_keycode(0x0062)][1<<2] = Proc.new do
        puts 'C-b pressed!'
      end
      result
    end

    private

    def to_keycode(keysym)
      XlibWrapper.keysym_to_keycode(@display, keysym)
    end
  end
end
