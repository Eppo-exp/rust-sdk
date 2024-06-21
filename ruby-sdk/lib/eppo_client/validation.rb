# frozen_string_literal: true

require_relative 'custom_errors'

# The helper module to validate keys
module EppoClient
  module_function

  def validate_not_blank(field_name, field_value)
    (field_value.nil? || field_value == '') && raise(
      EppoClient::InvalidValueError, "#{field_name} cannot be blank"
    )
  end
end
