MRuby::Gem::Specification.new('xkremap') do |spec|
  spec.license = 'MIT'
  spec.author  = 'Takashi Kokubun'
  spec.summary = 'Dynamic key remapper for X Window System'
  spec.bins    = ['xkremap']

  spec.add_dependency 'mruby-eval', core: 'mruby-eval'

  spec.add_dependency 'mruby-env',         mgem: 'mruby-env'
  spec.add_dependency 'mruby-io',          mgem: 'mruby-io'
  spec.add_dependency 'mruby-process',     mgem: 'mruby-process'
  spec.add_dependency 'mruby-onig-regexp', mgem: 'mruby-onig-regexp'
  spec.add_dependency 'mruby-shellwords',  mgem: 'mruby-shellwords'
end
