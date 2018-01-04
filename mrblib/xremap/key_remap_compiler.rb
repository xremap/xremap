module Xremap
  class KeyRemapCompiler
    def initialize(config, display)
      @config  = config
      @display = display
      puts "Config loaded: #{@config.inspect}"
    end

    # @return [Hash] : keycode(Fixnum) -> state(Fixnum) -> handler(Proc)
    def compile_for(window)
      result = Hash.new { |h, k| h[k] = {} }

      # guard segmentation fault
      return result if window == 0

      set_handlers_for(result, window)
      result
    end

    private

    def set_handlers_for(result, window)
      @config.remaps_for(@display, window).each do |remap|
        from = remap.from_key
        tos  = remap.to_keys

        actions = remap.to_keys.map do |to|
          case to.action
          when :input
            Proc.new { XlibWrapper.input_key(@display, to.keysym, to.modifier) }
          when :press
            Proc.new { XlibWrapper.press_key(@display, to.keysym, to.modifier) }
          when :release
            Proc.new { XlibWrapper.release_key(@display, to.keysym, to.modifier) }
          when :execute
            Proc.new { system("nohup /bin/sh -c #{to.command.shellescape} >/dev/null 2>&1 &") }
          else
            raise "unexpected action: #{to.action.inspect}"
          end
        end

        result[to_keycode(from.keysym)][from.modifier] =
          if actions.length == 1
            actions.first
          else
            Proc.new { actions.each { |action| action.call } }
          end
      end
    end

    def to_keycode(keysym)
      XlibWrapper.keysym_to_keycode(@display, keysym)
    end
  end
end
