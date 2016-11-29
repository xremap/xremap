module Xkremap
  class Config
    Key    = Struct.new(:keysym, :modifier)
    Remap  = Struct.new(:from_key, :to_keys)

    class Window < Struct.new(:class_only, :class_not)
      def class_only
        super ? Array(super) : []
      end

      def class_not
        super ? Array(super) : []
      end
    end

    AnyWindow = Window.new

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

    attr_reader :remaps_by_window

    def initialize
      @remaps_by_window = Hash.new { |h, k| h[k] = [] }
    end

    def remaps_for(display, window)
      klass = XlibWrapper.fetch_window_class(display, window)
      remaps_by_window[AnyWindow] + class_specific_remaps(klass)
    end

    private

    def class_specific_remaps(klass)
      @remaps_by_window.select do |window, _|
        if !window.class_only.empty?
          window.class_only.include?(klass)
        elsif !window.class_not.empty?
          !window.class_not.include?(klass)
        else
          false
        end
      end.map { |_, remaps| remaps }.flatten
    end
  end
end
