# frozen_string_literal: true

require_relative 'validation'
require_relative 'assignment_logger'

module EppoClient
  # The class for configuring the Eppo client singleton
  class Config
    attr_reader :api_key, :assignment_logger, :base_url

    def initialize(api_key, assignment_logger: AssignmentLogger.new, base_url: EppoClient::Core::DEFAULT_BASE_URL)
      @api_key = api_key
      @assignment_logger = assignment_logger
      @base_url = base_url
    end

    def validate
      EppoClient.validate_not_blank('api_key', @api_key)
    end

    # Hide instance variables (specifically api_key) from logs
    def inspect
      "#<EppoClient::Config:#{object_id}>"
    end
  end
end
