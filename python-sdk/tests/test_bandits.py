import pytest

import eppo_client
from eppo_client.assignment_logger import AssignmentLogger
from eppo_client.bandit import ContextAttributes, BanditResult

from .util import load_test_files, init

test_data = load_test_files("bandit-tests")


@pytest.fixture(scope="session", autouse=True)
def init_fixture():
    init("bandit")
    yield


@pytest.mark.parametrize("test_case", test_data, ids=lambda x: x["file_name"])
def test_get_bandit_action(test_case):
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

        assert result.variation == subject["assignment"]["variation"], (
            f"Flag {flag} failed for subject {subject['subjectKey']}:"
            f"expected assignment {subject['assignment']['variation']}, got {result.variation}"
        )
        assert result.action == subject["assignment"]["action"], (
            f"Flag {flag} failed for subject {subject['subjectKey']}:"
            f"expected action {subject['assignment']['action']}, got {result.action}"
        )


@pytest.mark.rust_only
@pytest.mark.parametrize("test_case", test_data, ids=lambda x: x["file_name"])
def test_get_bandit_action_details(test_case):
    client = eppo_client.get_instance()

    flag = test_case["flag"]
    default_value = test_case["defaultValue"]

    for subject in test_case["subjects"]:
        result = client.get_bandit_action_details(
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

        assert result.variation == subject["assignment"]["variation"], (
            f"Flag {flag} failed for subject {subject['subjectKey']}:"
            f"expected assignment {subject['assignment']['variation']}, got {result.variation}"
        )
        assert result.action == subject["assignment"]["action"], (
            f"Flag {flag} failed for subject {subject['subjectKey']}:"
            f"expected action {subject['assignment']['action']}, got {result.action}"
        )
