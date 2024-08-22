import pytest
import os
import json
from time import sleep

import eppo_client
from eppo_client.assignment_logger import AssignmentLogger
from eppo_client.bandit import ContextAttributes, BanditResult

TEST_DIR = os.path.join(
    os.path.dirname(os.path.abspath(__file__)),
    "../../../sdk-test-data/ufc/bandit-tests",
)
test_data = []
for file_name in os.listdir(TEST_DIR):
    with open("{}/{}".format(TEST_DIR, file_name)) as test_case_json:
        test_case_dict = json.load(test_case_json)
        test_case_dict["file_name"] = file_name
        test_data.append(test_case_dict)

MOCK_BASE_URL = "http://localhost:8378/"


@pytest.fixture(scope="session", autouse=True)
def init_fixture():
    eppo_client.init(
        eppo_client.config.Config(
            base_url=MOCK_BASE_URL + "bandit/api",
            api_key="dummy",
            assignment_logger=AssignmentLogger(),
        )
    )
    sleep(0.1)  # wait for initialization
    yield


@pytest.mark.parametrize("test_case", test_data, ids=lambda x: x["file_name"])
def test_bandit_generic_test_cases(test_case):
    client = eppo_client.get_instance()

    flag = test_case["flag"]
    default_value = test_case["defaultValue"]

    for subject in test_case["subjects"]:
        result = client.get_bandit_action(
            flag,
            subject["subjectKey"],
            ContextAttributes(
                numeric_attributes=subject["subjectAttributes"]["numericAttributes"],
                categorical_attributes=subject["subjectAttributes"][
                    "categoricalAttributes"
                ],
            ),
            {
                action["actionKey"]: ContextAttributes(
                    action["numericAttributes"], action["categoricalAttributes"]
                )
                for action in subject["actions"]
            },
            default_value,
        )

        expected_result = BanditResult(
            subject["assignment"]["variation"], subject["assignment"]["action"]
        )

        assert result.variation == subject["assignment"]["variation"], (
            f"Flag {flag} failed for subject {subject['subjectKey']}:"
            f"expected assignment {subject['assignment']['variation']}, got {result.variation}"
        )
        assert result.action == subject["assignment"]["action"], (
            f"Flag {flag} failed for subject {subject['subjectKey']}:"
            f"expected action {subject['assignment']['action']}, got {result.action}"
        )
