MRuby::Build.new do |conf|
  toolchain :gcc

  conf.gem File.expand_path(File.dirname(__FILE__))
  conf.instance_eval do
    # Allow showing backtrace.
    @mrbc.compile_options += ' -g'
  end
end
