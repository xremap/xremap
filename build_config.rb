MRuby::Build.new do |conf|
  toolchain :gcc

  conf.gembox 'default'
  conf.gem File.expand_path(File.dirname(__FILE__))

  conf.instance_eval do
    # Allow showing backtrace.
    @mrbc.compile_options += ' -g'
  end

  conf.cc do |cc|
    # Never support Visual C++.
    # https://github.com/mruby/mruby/blob/1.2.0/CONTRIBUTING.md#comply-with-c99-isoiec-98991999
    (cc.flags.first.is_a?(String) ? cc.flags : cc.flags.first).reject! do |flag|
      flag == '-Wdeclaration-after-statement'
    end
  end

  conf.linker do |linker|
    linker.libraries += %w(X11)
  end
end
