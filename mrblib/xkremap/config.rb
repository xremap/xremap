module Xkremap
  class Config
    # @param [String] filename
    def self.load(filename)
      unless File.exist?(filename)
        raise "Config file does not exist!: #{filename.inspect}"
        exit 1
      end

      config = self.new
      ConfigDSL.new(config).instance_eval(File.read(filename))
      config
    end

    attr_reader :remaps

    def initialize
      @remaps = []
    end

    Key   = Struct.new(:keysym, :modifier)
    Remap = Struct.new(:from_key, :to_keys)
  end
end
