from time import sleep
import pytest

import eppo_client
from eppo_client.config import Config, AssignmentLogger

from .util import init


def test_is_initialized_false():
    client = init("ufc", wait_for_init=False)
    assert client.is_initialized() == False


def test_is_initialized_true():
    client = init("ufc", wait_for_init=True)
    assert client.is_initialized() == True


@pytest.mark.rust_only
def test_wait_for_initialization():
    client = init("ufc", wait_for_init=False)
    client.wait_for_initialization()
    assert client.is_initialized() == True


def test_get_flag_keys_none():
    client = init("ufc", wait_for_init=False)
    assert client.get_flag_keys() == set()


def test_get_flag_keys_some():
    client = init("ufc", wait_for_init=True)

    keys = client.get_flag_keys()
    assert isinstance(keys, set)
    assert len(keys) != 0
    assert "numeric_flag" in keys


def test_get_bandit_keys_none():
    client = init("bandit", wait_for_init=False)
    assert client.get_bandit_keys() == set()


def test_get_bandit_keys_some():
    client = init("bandit", wait_for_init=True)

    keys = client.get_bandit_keys()
    assert isinstance(keys, set)
    assert len(keys) != 0
    assert "banner_bandit" in keys
