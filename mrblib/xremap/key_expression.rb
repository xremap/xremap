module Xremap
  module KeyExpression
    class << self
      # @param  [String] exp
      # @return [Xremap::Config::Key] key
      def compile(exp)
        keyexp, modifiers = split_into_key_and_mods(exp)
        Config::Key.new(to_keysym(keyexp), modifier_mask(modifiers))
      end

      private

      def split_into_key_and_mods(exp)
        modifiers = []
        while exp.match(/\A(?<modifier>(C|Ctrl|M|Alt|Shift|Super|Win))-/)
          modifier = Regexp.last_match[:modifier]
          modifiers << modifier
          exp = exp.sub(/\A#{modifier}-/, '')
        end
        [exp, modifiers]
      end

      def modifier_mask(modifiers)
        mask = X11::NoModifier
        modifiers.each do |modifier|
          case modifier
          when 'C', 'Ctrl'
            mask |= X11::ControlMask
          when 'M', 'Alt'
            mask |= X11::Mod1Mask
          when 'Super', 'Win'
            mask |= X11::Mod4Mask
          when 'Shift'
            mask |= X11::ShiftMask
          end
        end
        mask
      end

      def to_keysym(keyexp)
        if keyexp.start_with?('XF86XK_')
          X11.const_get(keyexp)
        else
          X11.const_get("XK_#{keyexp}")
        end
      end
    end
  end
end
