module Xremap
  class Config
    # FIXME: :to_keys should be :to_actions, and Key and Execute should be adapted.
    Remap = Struct.new(:from_key, :to_keys)
    Execute = Struct.new(:command, :action)

    class Key < Struct.new(:keysym, :modifier, :action)
      def initialize(*)
        super
        self.action ||= :input
      end
    end

    class Window < Struct.new(:class_only, :class_not)
      def initialize(*)
        super
        self.class_only = self.class_only ? Array(self.class_only) : []
        self.class_not  = self.class_not  ? Array(self.class_not)  : []
      end
    end

    AnyWindow = Window.new

    # @param [String] filename
    def self.load(filename)
      unless File.exist?(filename)
        raise "Config file does not exist!: #{filename.inspect}"
        exit 1
      end

      config_dir = File.dirname(File.expand_path(filename))
      config = self.new(config_dir)
      ConfigDSL.new(config).instance_eval(File.read(filename))
      config
    end

    attr_reader :remaps_by_window, :config_dir

    def initialize(config_dir)
      @remaps_by_window = Hash.new { |h, k| h[k] = [] }
      @config_dir = config_dir
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
