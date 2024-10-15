from eppo_client import Configuration

import pytest
import json

from .util import init

FLAGS_CONFIG = json.dumps({
    "createdAt": "2024-09-09T10:18:15.988Z",
    "environment": {"name": "test"},
    "flags": {}
}).encode('utf-8')

BANDITS_CONFIG = json.dumps({
    "bandit1": {"type": "some_type"},
    "bandit2": {"type": "another_type"}
}).encode('utf-8')

class TestConfiguration:
    def test_init_valid(self):
        Configuration(flags_configuration=FLAGS_CONFIG)

    def test_init_invalid_json(self):
        """Input is not valid JSON string."""
        with pytest.raises(Exception):
            Configuration(flags_configuration=b"{")

    def test_init_invalid_format(self):
        """flags is specified as array instead of object"""
        with pytest.raises(Exception):
            Configuration(
                flags_configuration=FLAGS_CONFIG
            )

@pytest.mark.rust_only
def test_configuration_none():
    client = init("ufc", wait_for_init=False)
    configuration = client.get_configuration()
    assert configuration == None

@pytest.mark.rust_only
def test_configuration_some():
    client = init("ufc", wait_for_init=True)
    configuration = client.get_configuration()
    assert configuration != None

    flag_config = configuration.get_flags_configuration()

    result = json.loads(flag_config)
    assert result["environment"] == {"name": "Test"}
    assert "numeric_flag" in result["flags"]

def test_bandit_configuration():
    # Initialize Configuration with both flags and bandits
    config = Configuration(flags_configuration=FLAGS_CONFIG, bandits_configuration=BANDITS_CONFIG)

    bandit_keys = config.get_bandit_keys()
    assert isinstance(bandit_keys, set)
    assert bandit_keys == {"bandit1", "bandit2"}
