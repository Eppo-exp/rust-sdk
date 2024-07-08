# frozen_string_literal: true

require 'json'

RSpec.describe EppoClient do
  it "has a version number" do
    expect(EppoClient::VERSION).not_to be nil
  end

  context "given a client with UFC test config" do
    client_with_test_config("ufc")

    Dir["../sdk-test-data/ufc/tests/*.json"].each do |file|
      basename = File.basename(file)
      context "with test file #{basename}", :file => basename do
        data = JSON.parse(File.read(file))

        flag_key = data["flag"]
        variation_type = data["variationType"]
        default_value = data["defaultValue"]

        data["subjects"].each do |subject|
          subject_key = subject["subjectKey"]
          subject_attributes = subject["subjectAttributes"]

          it "#{subject_key}", :subject => subject_key do
            result =
              case variation_type
              when "STRING"
                EppoClient::Client.instance.get_string_assignment(flag_key, subject_key, subject_attributes, default_value)
              when "NUMERIC"
                EppoClient::Client.instance.get_numeric_assignment(flag_key, subject_key, subject_attributes, default_value)
              when "INTEGER"
                EppoClient::Client.instance.get_integer_assignment(flag_key, subject_key, subject_attributes, default_value)
              when "BOOLEAN"
                EppoClient::Client.instance.get_boolean_assignment(flag_key, subject_key, subject_attributes, default_value)
              when "JSON"
                EppoClient::Client.instance.get_json_assignment(flag_key, subject_key, subject_attributes, default_value)
              else raise "unexpected variationType: #{variation_type}"
              end

            expect(result).to eq(subject["assignment"])
          end
        end
      end
    end
  end
end
