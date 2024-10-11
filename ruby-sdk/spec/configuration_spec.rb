# frozen_string_literal: true

# require 'eppo_client'

RSpec.describe EppoClient::Configuration do
  flags_config = File.read "../sdk-test-data/ufc/flags-v1.json"

  describe "new()" do
    it "initializes from flags_configuration" do
      EppoClient::Configuration.new(flags_configuration: flags_config)
    end

    it "initializes from flags_configuration and bandits_configuration" do
      flags_config = File.read "../sdk-test-data/ufc/bandit-flags-v1.json"
      bandits_config = File.read "../sdk-test-data/ufc/bandit-models-v1.json"
      EppoClient::Configuration.new(flags_configuration: flags_config, bandits_configuration: bandits_config)
    end

    it "requires flags_configuration to be a keyword" do
      expect {
        EppoClient::Configuration.new(flags_config)
      }.to raise_error TypeError
    end

    it "throws on parse error" do
      expect {
        EppoClient::Configuration.new(flags_configuration: '{"invalid": "configuration"}')
      }.to raise_error ArgumentError
    end

    it "accepts explicit nil as bandits_configuration" do
      EppoClient::Configuration.new(flags_configuration: flags_config, bandits_configuration: nil)
    end
  end

  describe "flags_configuration()" do
    it "returns configuration" do
      configuration = EppoClient::Configuration.new(flags_configuration: flags_config)

      flags = configuration.flags_configuration

      expect(flags).to be_a String
    end
  end

  describe "bandits_configuration()" do
    it "returns configuration" do
      flags_config = File.read "../sdk-test-data/ufc/bandit-flags-v1.json"
      bandits_config = File.read "../sdk-test-data/ufc/bandit-models-v1.json"
      configuration = EppoClient::Configuration.new(flags_configuration: flags_config, bandits_configuration: bandits_config)

      bandits = configuration.bandits_configuration

      expect(bandits).to be_a String
    end

    it "returns nil when there's no bandits" do
      configuration = EppoClient::Configuration.new(flags_configuration: flags_config)

      bandits = configuration.bandits_configuration

      expect(bandits).to be_nil
    end
  end

  it "can be reinstantiated from own configuration" do
    config1 = EppoClient::Configuration.new(flags_configuration: flags_config)

    config2 = EppoClient::Configuration.new(flags_configuration: config1.flags_configuration, bandits_configuration: config1.bandits_configuration)

    expect(config1.flags_configuration).to eq(config2.flags_configuration)
    expect(config1.bandits_configuration).to eq(config2.bandits_configuration)
  end

  it "can be reinstantiated from own configuration (with bandits)" do
    flags_config = File.read "../sdk-test-data/ufc/bandit-flags-v1.json"
    bandits_config = File.read "../sdk-test-data/ufc/bandit-models-v1.json"
    config1 = EppoClient::Configuration.new(flags_configuration: flags_config, bandits_configuration: bandits_config)

    config2 = EppoClient::Configuration.new(flags_configuration: config1.flags_configuration, bandits_configuration: config1.bandits_configuration)

    expect(config1.flags_configuration).to eq(config2.flags_configuration)
    # JSON parsing is an internal detail and is not a public
    # guarantee. We're using it here because serialization order of
    # bandits is not guaranteed.
    expect(JSON.parse(config1.bandits_configuration)).to eq(JSON.parse(config2.bandits_configuration))
  end
end
