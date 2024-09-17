from eppo_client import Configuration

import pytest
import json

from .util import init


class TestConfiguration:
    def test_init_valid(self):
        Configuration(
            flags_configuration=b'{"createdAt":"2024-09-09T10:18:15.988Z","environment":{"name":"test"},"flags":{}}'
        )

    def test_init_invalid_json(self):
        """Input is not valid JSON string."""
        with pytest.raises(Exception):
            Configuration(flags_configuration=b"{")

    def test_init_invalid_format(self):
        """flags is specified as array instead of object"""
        with pytest.raises(Exception):
            Configuration(
                flags_configuration=b'{"createdAt":"2024-09-09T10:18:15.988Z","environment":{"name":"test"},"flags":[]}'
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
