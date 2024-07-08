# frozen_string_literal: true

require "eppo_client"

def client_with_test_config(test_name)
    config = EppoClient::Config.new("test-api-key", base_url: "http://127.0.0.1:8378/#{test_name}/api")
    EppoClient::Client.instance.init(config)

    # Sleep to allow the client to fetch config
    sleep(0.5)
end

RSpec.configure do |config|
  # Enable flags like --only-failures and --next-failure
  config.example_status_persistence_file_path = ".rspec_status"

  # Disable RSpec exposing methods globally on `Module` and `main`
  config.disable_monkey_patching!

  config.expect_with :rspec do |c|
    c.syntax = :expect
  end
end
