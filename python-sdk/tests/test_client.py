from time import sleep
import pytest
import threading
import weakref

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


def test_shutdown_clean():
    client = init("ufc", wait_for_init=True)
    # Ensure client is fully initialized
    assert client.is_initialized() == True
    # Test clean shutdown
    client.shutdown()
    # Verify client is no longer initialized
    assert client.is_initialized() == False


def test_shutdown_during_operations():
    client = init("ufc", wait_for_init=True)
    
    # Start some concurrent operations
    futures = []
    for _ in range(5):
        future = threading.Thread(target=lambda: client.get_flag_keys())
        future.start()
        futures.append(future)
    
    # Shutdown while operations are in progress
    client.shutdown()
    
    # Verify operations either completed or failed gracefully
    for future in futures:
        future.join(timeout=1.0)
        assert not future.is_alive()
    
    assert client.is_initialized() == False


def test_shutdown_multiple_times():
    client = init("ufc", wait_for_init=True)
    # Should handle multiple shutdown calls gracefully
    client.shutdown()
    client.shutdown()  # Second shutdown should be no-op
    assert client.is_initialized() == False


@pytest.mark.parametrize("wait_for_init", [True, False])
def test_shutdown_at_different_states(wait_for_init):
    client = init("ufc", wait_for_init=wait_for_init)
    # Should handle shutdown regardless of initialization state
    client.shutdown()
    assert client.is_initialized() == False


def test_client_cleanup_on_delete():
    client = init("ufc", wait_for_init=True)
    # Get the client id for checking after deletion
    client_id = id(client)
