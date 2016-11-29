module Xkremap
  class ConfigDSL
    # @param [Xkremap::Config] config
    def initialize(config)
      @config = config
    end

    def remap(from_str, options = {})
      to_strs = Array(options.fetch(:to))
      @config.remaps << Config::Remap.new(
        compile_exp(from_str),
        to_strs.map { |str| compile_exp(str) }
      )
    end

    private

    def compile_exp(str)
      KeyExpression.compile(str)
    end
  end
end
