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

    private

    def compile_exp(str)
      KeyExpression.compile(str)
    end
  end
end
