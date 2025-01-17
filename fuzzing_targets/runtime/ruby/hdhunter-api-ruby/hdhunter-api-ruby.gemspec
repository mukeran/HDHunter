# frozen_string_literal: true

require_relative "lib/hdhunter/version"

Gem::Specification.new do |spec|
  spec.name = "hdhunter-api-ruby"
  spec.version = HDHunter::VERSION
  spec.authors = ["Keran Mu"]
  spec.email = ["mukeran@mukeran.com"]

  spec.summary = "Ruby bindings for the HDHunter API."
  spec.description = "TRuby bindings for the HDHunter API."
  spec.homepage = "https://git.var.codes:4443/http_ambiguous/HDHunter"
  spec.required_ruby_version = ">= 3.0.0"

  spec.metadata["allowed_push_host"] = "https://git.var.codes:4443/api/packages/mukeran/rubygems"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://git.var.codes:4443/http_ambiguous/hdhunter-targets"

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  gemspec = File.basename(__FILE__)
  spec.files = IO.popen(%w[git ls-files -z], chdir: __dir__, err: IO::NULL) do |ls|
    ls.readlines("\x0", chomp: true).reject do |f|
      (f == gemspec) ||
        f.start_with?(*%w[bin/ test/ spec/ features/ .git appveyor Gemfile])
    end
  end
  spec.require_paths = ["lib"]

  # Uncomment to register a new dependency of your gem
  spec.add_dependency "ffi"

  # For more information and examples about making a new gem, check out our
  # guide at: https://bundler.io/guides/creating_gem.html
end
