module Xkremap
  class Config
    # @param [String] dsl
    def self.load(dsl)
      config = self.new
      ConfigContext.new(config).instance_eval(dsl)
      config
    end

    def initialize
      @remaps = []
    end

    attr_reader :remaps
  end
end
