# frozen_string_literal: true

require_relative "eppo_client/client"
require_relative "eppo_client/version"

module EppoClient
  def init(config)
    client = EppoClient::Client.instance
    client.init(config)
  end

  module_function :init
end
