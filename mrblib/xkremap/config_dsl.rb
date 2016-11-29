module Xkremap
  class ConfigDSL
    # @param [Xkremap::Config] config
    def initialize(config, win = Config::AnyWindow)
      @config = config
      @window = win
    end

    def remap(from_str, options = {})
      to_strs = Array(options.fetch(:to))
      @config.remaps_by_window[@window] << Config::Remap.new(
        compile_exp(from_str),
        to_strs.map { |str| compile_exp(str) }
      )
    end

    def window(options = {}, &block)
      win = Config::Window.new(options[:class_only], options[:class_not])
      ConfigDSL.new(@config, win).instance_exec(&block)
    end

    def press(str)
      key = compile_exp(str)
      key.action = :press
      key
    end

    def release(str)
      key = compile_exp(str)
      key.action = :release
      key
    end

    private

    def compile_exp(exp)
      if exp.is_a?(Config::Key)
        exp
      elsif exp.is_a?(String)
        KeyExpression.compile(exp)
      else
        raise "unexpected expression: #{exp.inspect}"
      end
    end
  end
end
