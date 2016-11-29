module Xkremap
  class KeyRemapBuilder
    # @attribute [Hash] prebuilt
    #   [Fixnum] keycode -> [Fixnum] state -> [Proc] handler
    attr_reader :prebuilt

    # @param [Xkremap::Config] config
    # @param [Xkremap::Display] display
    def initialize(config, display)
      @config   = config
      @display  = display
      @prebuilt = Hash.new { |h, k| h[k] = {} }
      @current_window = fetch_active_window
    end

    def build
      puts 'rebuilt!'
    end

    def active_window_changed?
      next_window = fetch_active_window
      @current_window != next_window
    ensure
      @current_window = next_window
    end

    private

    def fetch_active_window
      XlibWrapper.fetch_active_window(@display)
    end
  end
end
