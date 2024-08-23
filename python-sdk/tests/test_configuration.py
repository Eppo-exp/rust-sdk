import pytest
import json

from .util import init


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
