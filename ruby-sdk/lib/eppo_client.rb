# frozen_string_literal: true

require_relative "eppo_client/client"
require_relative "eppo_client/version"

# EppoClient is the main module for initializing the Eppo client.
# It provides a method to initialize the client with a given configuration.
module EppoClient
  def init(config)
    client = EppoClient::Client.instance
    client.init(config)
  end

  module_function :init
end
