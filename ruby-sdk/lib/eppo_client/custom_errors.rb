# frozen_string_literal: true

module EppoClient
  # A custom error class for AssignmentLogger
  class AssignmentLoggerError < StandardError
    def initialize(message)
      super("AssignmentLoggerError: #{message}")
    end
  end
end
