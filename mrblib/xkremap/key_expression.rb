module Xkremap
  module KeyExpression
    class << self
      # @param  [String] exp
      # @return [Xkremap::Config::Key] key
      def compile(exp)
        case exp
        when /\A(?<keyexp>[^-]+)\z/
          Config::Key.new(to_keysym(Regexp.last_match[:keyexp]), X11::NoModifier)
        when /\AC-(?<keyexp>[^-]+)\z/
          Config::Key.new(to_keysym(Regexp.last_match[:keyexp]), X11::ControlMask)
        when /\A(M|Alt)-(?<keyexp>[^-]+)\z/
          Config::Key.new(to_keysym(Regexp.last_match[:keyexp]), X11::Mod1Mask)
        else
          raise "unexpected key expression pattern!: #{exp.inspect}"
        end
      end

      private

      def to_keysym(keyexp)
        X11.const_get(x11_const_name(keyexp))
      end

      def x11_const_name(keyexp)
        if keyexp.length == 1
          "XK_#{keyexp.downcase}"
        else
          "XK_#{capitalize(keyexp)}"
        end
      end

      def capitalize(str)
        result = str.downcase
        result[0] = str[0].upcase
        result
      end
    end
  end
end
