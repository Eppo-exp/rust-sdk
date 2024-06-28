# frozen_string_literal: true

RSpec.describe EppoClient do
  it "has a version number" do
    expect(EppoClient::VERSION).not_to be nil
  end

  context "given config" do
    # config = EppoClient::Config.new("test_api_key")
    config = EppoClient::Config.new(ENV.fetch("EPPO_API_KEY"))

    it "can be initialized" do
      EppoClient::Client.instance.init(config)
    end

    it "can get boolean assignment" do
      sleep(2)

      value = EppoClient::Client.instance.get_boolean_assignment("a-boolean-flag", "subject5", {}, false)

      puts value

      expect(value).to be(true)
    end
  end
end
