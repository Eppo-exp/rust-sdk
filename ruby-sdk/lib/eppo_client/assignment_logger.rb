# frozen_string_literal: true

require_relative "custom_errors"

module EppoClient
  # The base assignment logger class to override
  class AssignmentLogger
    def log_assignment(_assignment_event)
      raise(EppoClient::AssignmentLoggerError, "log_assignment has not been set up")
    end
  end
end
