import pytest

from eppo_client.config import Config
from eppo_client.assignment_logger import AssignmentLogger


class TestConfig:
    def test_can_create_with_api_key_and_assignment_logger(self):
        Config(api_key="test-key", assignment_logger=AssignmentLogger())

    @pytest.mark.rust_only
    def test_requires_non_empty_key(self):
        with pytest.raises(ValueError):
            Config(api_key="", assignment_logger=AssignmentLogger())

    def test_requires_api_key(self):
        # Python SDK raises Pydantic's ValidationError.
        # Python-on-rust raises TypeError.
        with pytest.raises(Exception):
            Config(assignment_logger=AssignmentLogger())

    def test_requires_assignment_logger(self):
        # Python SDK raises Pydantic's ValidationError.
        # Python-on-rust raises TypeError.
        with pytest.raises(Exception):
            Config(api_key="test-key")

    def test_assignment_logger_must_be_a_subclass_of_logger(self):
        class MyLogger:
            pass

        # Python SDK raises Pydantic's ValidationError.
        # Python-on-rust raises TypeError.
        with pytest.raises(Exception):
            Config(api_key="test-key", assignment_logger=MyLogger())

    def test_assignment_accepts_a_subclass_of_logger(self):
        class MyLogger(AssignmentLogger):
            pass

        Config(api_key="test-key", assignment_logger=MyLogger())

    # This one is failing on native python sdk as we don't have
    # `validate_assignment` enabled.
    @pytest.mark.rust_only
    def test_cant_reset_assignment_logger(self):
        config = Config(api_key="test-key", assignment_logger=AssignmentLogger())
        with pytest.raises(TypeError):
            config.assignment_logger = None
        assert config.assignment_logger is not None

    def test_can_set_assignment_logger_to_another_logger(self):
        config = Config(api_key="test-key", assignment_logger=AssignmentLogger())
        config.assignment_logger = AssignmentLogger()

    @pytest.mark.rust_only
    def test_poll_interval_seconds_cannot_be_0(self):
        with pytest.raises(ValueError):
            Config(
                api_key="test-key",
                assignment_logger=AssignmentLogger(),
                poll_interval_seconds=0,
            )
