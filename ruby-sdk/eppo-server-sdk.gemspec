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
  spec.required_rubygems_version = ">= 3.3.11"

  spec.metadata = {
    "bug_tracker_uri" => "https://github.com/Eppo-exp/ruby-sdk/issues",
    "documentation_uri" => "https://docs.geteppo.com/feature-flags/sdks/server-sdks/ruby/",
    "homepage_uri" => "https://geteppo.com/",
    "source_code_uri" => "https://github.com/Eppo-exp/ruby-sdk",
    "wiki_uri" => "https://github.com/Eppo-exp/ruby-sdk/wiki"
  }

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  gemspec = File.basename(__FILE__)
  spec.files = IO.popen(%w[git ls-files -z], chdir: __dir__, err: IO::NULL) do |ls|
    ls.readlines("\x0", chomp: true).reject do |f|
      (f == gemspec) ||
        f.start_with?(*%w[bin/ test/ spec/ features/ .cargo/ .git/ .github/ appveyor Gemfile])
    end
  end
  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/eppo_rb/Cargo.toml"]

  # Uncomment to register a new dependency of your gem
  # spec.add_dependency "example-gem", "~> 1.0"

  # For more information and examples about making a new gem, check out our
  # guide at: https://bundler.io/guides/creating_gem.html
end
