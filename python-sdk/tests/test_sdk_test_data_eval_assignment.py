import pytest

import eppo_client
from eppo_client.assignment_logger import AssignmentLogger

from .util import load_test_files, init

test_data = load_test_files("tests")


@pytest.fixture(scope="session", autouse=True)
def init_fixture():
    init("ufc")
    yield


@pytest.mark.parametrize("test_case", test_data, ids=lambda x: x["file_name"])
def test_assign_subject_in_sample(test_case):
    client = eppo_client.get_instance()
    print("---- Test case for {} Experiment".format(test_case["flag"]))

    get_typed_assignment = {
        "STRING": client.get_string_assignment,
        "INTEGER": client.get_integer_assignment,
        "NUMERIC": client.get_numeric_assignment,
        "BOOLEAN": client.get_boolean_assignment,
        "JSON": client.get_json_assignment,
    }[test_case["variationType"]]

    assignments = get_assignments(test_case, get_typed_assignment)
    for subject, assigned_variation in assignments:
        assert (
            assigned_variation == subject["assignment"]
        ), f"expected <{subject['assignment']}> for subject {subject['subjectKey']}, found <{assigned_variation}>"


@pytest.mark.parametrize("test_case", test_data, ids=lambda x: x["file_name"])
@pytest.mark.rust_only
def test_eval_details(test_case):
    client = eppo_client.get_instance()
    print("---- Test case for {} Experiment".format(test_case["flag"]))

    get_typed_assignment = {
        "STRING": client.get_string_assignment_details,
        "INTEGER": client.get_integer_assignment_details,
        "NUMERIC": client.get_numeric_assignment_details,
        "BOOLEAN": client.get_boolean_assignment_details,
        "JSON": client.get_json_assignment_details,
    }[test_case["variationType"]]

    assignments = get_assignments(test_case, get_typed_assignment)
    for subject, assigned_variation in assignments:
        assert (
            assigned_variation.variation == subject["assignment"]
        ), f"expected <{subject['assignment']}> for subject {subject['subjectKey']}, found <{assigned_variation}>"


def get_assignments(test_case, get_assignment_fn):
    # client = eppo_client.get_instance()
    # client.__is_graceful_mode = False

    print(test_case["flag"])
    assignments = []
    for subject in test_case.get("subjects", []):
        variation = get_assignment_fn(
            test_case["flag"],
            subject["subjectKey"],
            subject["subjectAttributes"],
            test_case["defaultValue"],
        )
        assignments.append((subject, variation))
    return assignments
