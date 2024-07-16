# frozen_string_literal: true

module EppoClient
  # A custom error class for AssignmentLogger
  class AssignmentLoggerError < StandardError
    def initialize(message)
      super("AssignmentLoggerError: #{message}")
    end
  end

  # A custom error class for invalid values
  class InvalidValueError < StandardError
    def initialize(message)
      super("InvalidValueError: #{message}")
    end
  end
end
