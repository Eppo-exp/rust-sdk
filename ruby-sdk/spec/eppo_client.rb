# frozen_string_literal: true

RSpec.describe EppoClient do
  it "has a version number" do
    expect(EppoClient::VERSION).not_to be nil
  end
end
