from time import sleep
import pytest

import eppo_client
from eppo_client.config import Config, AssignmentLogger


def init(suite="ufc"):
    return eppo_client.init(
        Config(
            api_key="blah",
            base_url=f"http://localhost:8378/{suite}/api",
            assignment_logger=AssignmentLogger(),
        )
    )


def wait_for_initialization():
    client = eppo_client.get_instance()
    if hasattr(client, "wait_for_initialization"):
        client.wait_for_initialization()
    elif not client.is_initialized():
        sleep(0.1)


def test_is_initialized_false():
    client = init()
    assert client.is_initialized() == False


def test_is_initialized_true():
    client = init()
    wait_for_initialization()
    assert client.is_initialized() == True


@pytest.mark.rust_only
def test_wait_for_initialization():
    client = init()
    client.wait_for_initialization()
    assert client.is_initialized() == True


def test_get_flag_keys_none():
    client = init()
    assert client.get_flag_keys() == set()


def test_get_flag_keys_some():
    client = init()
    wait_for_initialization()

    keys = client.get_flag_keys()
    assert isinstance(keys, set)
    assert len(keys) != 0
    assert "numeric_flag" in keys


def test_get_bandit_keys_none():
    client = init("bandit")
    assert client.get_bandit_keys() == set()


def test_get_bandit_keys_some():
    client = init("bandit")
    wait_for_initialization()

    keys = client.get_bandit_keys()
    assert isinstance(keys, set)
    assert len(keys) != 0
    assert "banner_bandit" in keys
