import pytest

from eppo_client import Config, AssignmentLogger


class TestConfig:
    def test_can_create_with_api_key_and_assignment_logger(self):
        Config(api_key="test-key", assignment_logger=AssignmentLogger())

    def test_requires_non_empty_key(self):
        with pytest.raises(ValueError):
            Config(api_key="", assignment_logger=AssignmentLogger())

    def test_requires_api_key(self):
        with pytest.raises(TypeError):
            Config(assignment_logger=AssignmentLogger())

    def test_requires_assignment_logger(self):
        with pytest.raises(TypeError):
            Config(api_key="test-key")

    def test_assignment_logger_must_be_a_subclass_of_logger(self):
        class MyLogger:
            pass

        with pytest.raises(TypeError):
            Config(api_key="test-key", assignment_logger=MyLogger())

    def test_assignment_accepts_a_subclass_of_logger(self):
        class MyLogger(AssignmentLogger):
            pass

        Config(api_key="test-key", assignment_logger=MyLogger())

    def test_cant_reset_assignment_logger(self):
        config = Config(api_key="test-key", assignment_logger=AssignmentLogger())
        with pytest.raises(TypeError):
            config.assignment_logger = None
