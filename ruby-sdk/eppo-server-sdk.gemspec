# frozen_string_literal: true

require_relative "lib/eppo_client/version"

Gem::Specification.new do |spec|
  spec.name = "eppo-server-sdk"
  spec.version = EppoClient::VERSION
  spec.authors = ["Eppo"]
  spec.email = ["eppo-team@geteppo.com"]

  spec.summary = "Eppo SDK for Ruby"
  spec.homepage = "https://github.com/Eppo-exp/ruby-sdk"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.0.0"
  # RubyGems 3.3.11 is the first version with Rust support:
  # https://blog.rubygems.org/2022/04/07/3.3.11-released.html
  spec.required_rubygems_version = ">= 3.3.11"

  spec.metadata = {
    "bug_tracker_uri" => "https://github.com/Eppo-exp/ruby-sdk/issues",
    "documentation_uri" => "https://docs.geteppo.com/feature-flags/sdks/server-sdks/ruby/",
    "homepage_uri" => "https://geteppo.com/",
    "source_code_uri" => "https://github.com/Eppo-exp/ruby-sdk",
    "wiki_uri" => "https://github.com/Eppo-exp/ruby-sdk/wiki"
  }

  spec.files = Dir["{lib,ext}/**/*", "LICENSE", "README.md", "Cargo.*"]
  spec.files.reject! { |f| File.directory?(f) }
  spec.files.reject! { |f| f =~ /\.(dll|so|dylib|lib|bundle)\Z/ }
  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/eppo_client/extconf.rb"]

  spec.add_dependency "rb_sys", "~> 0.9.102"

  # For more information and examples about making a new gem, check out our
  # guide at: https://bundler.io/guides/creating_gem.html
end
