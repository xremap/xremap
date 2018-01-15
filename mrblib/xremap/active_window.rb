module Xremap
  class ActiveWindow
    # @param [Fixnum] current_window
    attr_reader :current_window

    # @param [Xremap::Display] display
    def initialize(display)
      @display = display
      @current_window = fetch_active_window
    end

    def changed?
      next_window = fetch_active_window
      @current_window != next_window
    ensure
      @current_window = next_window
    end

    private

    def fetch_active_window
      sleep ENV.fetch('XREMAP_DELAY', '0.1').to_f
      XlibWrapper.fetch_active_window(@display)
    end
  end
end
