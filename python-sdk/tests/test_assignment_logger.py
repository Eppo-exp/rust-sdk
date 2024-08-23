from typing import Dict

from eppo_client.assignment_logger import AssignmentLogger


def test_can_inherit_assignment_logger():
    class MyAssignmentLogger(AssignmentLogger):
        pass

    logger = MyAssignmentLogger()


def test_can_override_methods():
    class MyAssignmentLogger(AssignmentLogger):
        def log_assignment(self, assignment_event: Dict):
            print("log_assignment", assignment_event)

        def log_bandit_action(self, bandit_event: Dict):
            print("log_assignment", bandit_event)

    logger = MyAssignmentLogger()


def test_has_log_assignment():
    AssignmentLogger().log_assignment({})


def test_has_log_bandit_action():
    AssignmentLogger().log_bandit_action({})
