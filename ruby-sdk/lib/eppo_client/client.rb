# frozen_string_literal: true

require 'singleton'
require 'logger'

require_relative 'config'
require_relative 'eppo_rb'

module EppoClient
  # The main client singleton
  class Client
    include Singleton
    attr_accessor :assignment_logger

    def init(config)
      config.validate

      @assignment_logger = config.assignment_logger
      @core = EppoClient::Core::Client.new(config)
    end

    def shutdown
      @core.shutdown
    end

    def get_string_assignment(flag_key, subject_key, subject_attributes, default_value)
      return get_assignment_inner(flag_key, subject_key, subject_attributes, "STRING", default_value)
    end

    def get_numeric_assignment(flag_key, subject_key, subject_attributes, default_value)
      return get_assignment_inner(flag_key, subject_key, subject_attributes, "NUMERIC", default_value)
    end

    def get_integer_assignment(flag_key, subject_key, subject_attributes, default_value)
      return get_assignment_inner(flag_key, subject_key, subject_attributes, "INTEGER", default_value)
    end

    def get_boolean_assignment(flag_key, subject_key, subject_attributes, default_value)
      return get_assignment_inner(flag_key, subject_key, subject_attributes, "BOOLEAN", default_value)
    end

    def get_json_assignment(flag_key, subject_key, subject_attributes, default_value)
      return get_assignment_inner(flag_key, subject_key, subject_attributes, "JSON", default_value)
    end

    def get_assignment_inner(flag_key, subject_key, subject_attributes, expected_type, default_value)
      logger = Logger.new($stdout)
      begin
        assignment = @core.get_assignment(flag_key, subject_key, subject_attributes, expected_type)

        event = assignment[:event]
        if event then
          begin
            event["metaData"]["sdkName"] = "ruby"
            event["metaData"]["sdkVersion"] = EppoClient::VERSION

            @assignment_logger.log_assignment(event)
          rescue EppoClient::AssignmentLoggerError
            # Error means log_assignment was not set up. This is okay to ignore.
          rescue => error
            logger.error("[Eppo SDK] Error logging assignment event: #{error}")
          end
        end

        value = assignment[:value]&.[](expected_type)
        return value.nil? ? default_value : value
      rescue => error
        logger.debug('[Eppo SDK] Failed to get assignment: #{error}')

        # TODO: graceful mode?
        return default_value
      end
    end

    private :get_assignment_inner
  end
end
