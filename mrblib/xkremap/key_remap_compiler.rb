module Xkremap
  class KeyRemapCompiler
    def initialize(config, display)
      @config  = config
      @display = display
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

      # C-b -> Left
      result[to_keycode(X11::XK_b)][X11::ControlMask] = Proc.new do
        XlibWrapper.input_key(display, X11::XK_Left, X11::NoModifier)
      end
    end

    def to_keycode(keysym)
      XlibWrapper.keysym_to_keycode(@display, keysym)
    end
  end
end
