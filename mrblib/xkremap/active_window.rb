module Xkremap
  class ActiveWindow
    # @param [Fixnum] current_window
    attr_reader :current_window

    # @param [Xkremap::Display] display
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
      XlibWrapper.fetch_active_window(@display)
    end
  end
end
