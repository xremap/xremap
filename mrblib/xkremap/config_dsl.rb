module Xkremap
  class ConfigDSL
    # @param [Xkremap::Config] config
    def initialize(config, win = Config::AnyWindow)
      @config = config
      @window = win
    end

    def remap(from_str, options = {})
      # Array() doesn't work for Config::Execute somehow.
      to_strs = options.fetch(:to)
      to_strs = [to_strs] unless to_strs.is_a?(Array)

      @config.remaps_by_window[@window] << Config::Remap.new(
        compile_exp(from_str),
        to_strs.map { |str| compile_exp(str) }
      )
    end

    def window(options = {}, &block)
      win = Config::Window.new(options[:class_only], options[:class_not])
      ConfigDSL.new(@config, win).instance_exec(&block)
    end

    def execute(str)
      Config::Execute.new(str, :execute)
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

    def define(name, &block)
      ConfigDSL.define_method(name, &block)
    end

    private

    def compile_exp(exp)
      case exp
      when Config::Key, Config::Execute
        exp
      when String
        KeyExpression.compile(exp)
      else
        raise "unexpected expression: #{exp.inspect}"
      end
    end
  end
end
