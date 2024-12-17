# frozen_string_literal: true

require "singleton"
require "logger"

require_relative "config"

# Tries to require the extension for the current Ruby version first
begin
  RUBY_VERSION =~ /(\d+\.\d+)/
  require_relative "#{Regexp.last_match(1)}/eppo_client"
rescue LoadError
  require_relative "eppo_client"
end

module EppoClient
  # The main client singleton
  class Client
    include Singleton
    attr_accessor :assignment_logger

    def init(config)
      config.validate

      if !@core.nil?
        STDERR.puts "Eppo Warning: multiple initialization of the client"
        @core.shutdown
      end

      @assignment_logger = config.assignment_logger
      @core = EppoClient::Core::Client.new(config)
    end

    def configuration
      @core.configuration
    end

    def configuration=(configuration)
      @core.configuration = configuration
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

    def get_string_assignment_details(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_details_inner(flag_key, subject_key, subject_attributes, "STRING", default_value)
    end

    def get_numeric_assignment_details(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_details_inner(flag_key, subject_key, subject_attributes, "NUMERIC", default_value)
    end

    def get_integer_assignment_details(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_details_inner(flag_key, subject_key, subject_attributes, "INTEGER", default_value)
    end

    def get_boolean_assignment_details(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_details_inner(flag_key, subject_key, subject_attributes, "BOOLEAN", default_value)
    end

    def get_json_assignment_details(flag_key, subject_key, subject_attributes, default_value)
      get_assignment_details_inner(flag_key, subject_key, subject_attributes, "JSON", default_value)
    end

    def get_bandit_action(flag_key, subject_key, subject_attributes, actions, default_variation)
      attributes = coerce_context_attributes(subject_attributes)
      actions = actions.to_h { |action, attributes| [action, coerce_context_attributes(attributes)] }
      result = @core.get_bandit_action(flag_key, subject_key, attributes, actions, default_variation)

      log_assignment(result[:assignment_event])
      log_bandit_action(result[:bandit_event])

      return {:variation => result[:variation], :action => result[:action]}
    end

    def get_bandit_action_details(flag_key, subject_key, subject_attributes, actions, default_variation)
      attributes = coerce_context_attributes(subject_attributes)
      actions = actions.to_h { |action, attributes| [action, coerce_context_attributes(attributes)] }
      result, details = @core.get_bandit_action_details(flag_key, subject_key, attributes, actions, default_variation)

      log_assignment(result[:assignment_event])
      log_bandit_action(result[:bandit_event])

      return {
        :variation => result[:variation],
        :action => result[:action],
        :evaluationDetails => details
      }
    end

    private

    # rubocop:disable Metrics/MethodLength
    def get_assignment_inner(flag_key, subject_key, subject_attributes, expected_type, default_value)
      logger = Logger.new($stdout)
      begin
        assignment = @core.get_assignment(flag_key, subject_key, subject_attributes, expected_type)
        if not assignment then
          return default_value
        end

        log_assignment(assignment[:event])

        return assignment[:value]
      rescue StandardError => error
        logger.debug("[Eppo SDK] Failed to get assignment: #{error}")

        # TODO: non-graceful mode?
        default_value
      end
    end
    # rubocop:enable Metrics/MethodLength

    # rubocop:disable Metrics/MethodLength
    def get_assignment_details_inner(flag_key, subject_key, subject_attributes, expected_type, default_value)
      result, event = @core.get_assignment_details(flag_key, subject_key, subject_attributes, expected_type)
      log_assignment(event)

      if not result[:variation] then
        result[:variation] = default_value
      end

      return result
    end
    # rubocop:enable Metrics/MethodLength

    def log_assignment(event)
      if not event then return end

      # Because rust's AssignmentEvent has a #[flatten] extra_logging
      # field, serde_magnus serializes it as a normal HashMap with
      # string keys.
      #
      # Convert keys to symbols here, so that logger sees symbol-keyed
      # events for both flag assignment and bandit actions.
      event = event.to_h { |key, value| [key.to_sym, value]}

      begin
        @assignment_logger.log_assignment(event)
      rescue EppoClient::AssignmentLoggerError
      # Error means log_assignment was not set up. This is okay to ignore.
      rescue StandardError => error
        logger = Logger.new($stdout)
        logger.error("[Eppo SDK] Error logging assignment event: #{error}")
      end
    end

    def log_bandit_action(event)
      if not event then return end

      begin
        @assignment_logger.log_bandit_action(event)
      rescue EppoClient::AssignmentLoggerError
      # Error means log_assignment was not set up. This is okay to ignore.
      rescue StandardError => error
        logger = Logger.new($stdout)
        logger.error("[Eppo SDK] Error logging bandit action event: #{error}")
      end
    end

    def coerce_context_attributes(attributes)
      numeric_attributes = attributes[:numeric_attributes] || attributes["numericAttributes"]
      categorical_attributes = attributes[:categorical_attributes] || attributes["categoricalAttributes"]
      if numeric_attributes || categorical_attributes then
        {
          numericAttributes: numeric_attributes.to_h do |key, value|
            value.is_a?(Numeric) ? [key, value] : [nil, nil]
          end.compact,
          categoricalAttributes: categorical_attributes.to_h do |key, value|
            value.nil? ? [nil, nil] : [key, value.to_s]
          end.compact,
        }
      end
    end
  end
end
