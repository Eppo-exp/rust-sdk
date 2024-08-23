from time import sleep
import pytest

import eppo_client
from eppo_client.config import Config, AssignmentLogger


def init():
    return eppo_client.init(
        Config(
            api_key="blah",
            base_url="http://localhost:8378/ufc/api",
            assignment_logger=AssignmentLogger(),
        )
    )


def test_is_initialized_false():
    client = init()
    assert client.is_initialized() == False


def test_is_initialized_true():
    client = init()
    sleep(0.1)
    assert client.is_initialized() == True


@pytest.mark.rust_only
def test_wait_for_initialization():
    client = init()
    client.wait_for_initialization()
    assert client.is_initialized() == True
