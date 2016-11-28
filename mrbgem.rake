MRuby::Gem::Specification.new('xkremap') do |spec|
  spec.license = 'MIT'
  spec.author  = 'Takashi Kokubun'
  spec.summary = 'Dynamic key remapper for X Window System'
  spec.bins    = ['xkremap']

  spec.add_dependency 'mruby-eval', core: 'mruby-eval'
end
