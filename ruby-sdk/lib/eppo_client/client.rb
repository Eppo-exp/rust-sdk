# frozen_string_literal: true

require "singleton"
require "logger"

require_relative "config"
require_relative "eppo_rb"

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
      get_assignment_inner(flag_key, subject_key, subject_attributes, "STRING", default_value)
    end

    def get_numeric_assignment(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_inner(flag_key, subject_key, subject_attributes, "NUMERIC", default_value)
    end

    def get_integer_assignment(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_inner(flag_key, subject_key, subject_attributes, "INTEGER", default_value)
    end

    def get_boolean_assignment(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_inner(flag_key, subject_key, subject_attributes, "BOOLEAN", default_value)
    end

    def get_json_assignment(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_inner(flag_key, subject_key, subject_attributes, "JSON", default_value)
    end

    # rubocop:disable Metrics/MethodLength
    def get_assignment_inner(flag_key, subject_key, subject_attributes, expected_type, default_value)
      logger = Logger.new($stdout)
      begin
        assignment = @core.get_assignment(flag_key, subject_key, subject_attributes, expected_type)
        if not assignment then
          return default_value
        end

        log_assignment(assignment[:event])

        return assignment[:value][expected_type]
      rescue StandardError => error
        logger.debug("[Eppo SDK] Failed to get assignment: #{error}")

        # TODO: non-graceful mode?
        default_value
      end
    end
    # rubocop:enable Metrics/MethodLength

    def get_bandit_action(flag_key, subject_key, subject_attributes, actions, default_variation)
      result = @core.get_bandit_action(flag_key, subject_key, subject_attributes, actions, default_variation)

      log_assignment(result[:assignment_event])
      log_bandit_action(result[:bandit_event])

      return {:variation => result[:variation], :action=>result[:action]}
    end

    def log_assignment(event)
      if not event then return end
      begin
        event["metaData"]["sdkName"] = "ruby"
        event["metaData"]["sdkVersion"] = EppoClient::VERSION

        @assignment_logger.log_assignment(event)
      rescue EppoClient::AssignmentLoggerError
      # Error means log_assignment was not set up. This is okay to ignore.
      rescue StandardError => error
        logger.error("[Eppo SDK] Error logging assignment event: #{error}")
      end
    end

    def log_bandit_action(event)
      if not event then return end
      begin
        event["metaData"]["sdkName"] = "ruby"
        event["metaData"]["sdkVersion"] = EppoClient::VERSION

        @assignment_logger.log_bandit_action(event)
      rescue EppoClient::AssignmentLoggerError
      # Error means log_assignment was not set up. This is okay to ignore.
      rescue StandardError => error
        logger.error("[Eppo SDK] Error logging bandit action event: #{error}")
      end
    end

    private :get_assignment_inner, :log_assignment, :log_bandit_action
  end
end
