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
        "updatedAt": "2023-09-13T04:52:06.462Z",
        "environment": {
            "name": "Test"
        },
        "bandits": {
            "car_bandit": {
                "banditKey": "car_bandit",
                "modelName": "falcon",
                "updatedAt": "2023-09-13T04:52:06.462Z",
                "modelVersion": "v456",
                "modelData": {
                    "gamma": 1.0,
                    "defaultActionScore": 5.0,
                    "actionProbabilityFloor": 0.2,
                    "coefficients": {
                        "toyota": {
                            "actionKey": "toyota",
                            "intercept": 1.0,
                            "actionNumericCoefficients": [{
                                "attributeKey": "speed",
                                "coefficient": 1,
                                "missingValueCoefficient": 0.0
                            }],
                            "actionCategoricalCoefficients": [],
                            "subjectNumericCoefficients": [],
                            "subjectCategoricalCoefficients": []
                        }
                    }
                }
            }
        }
    }).encode('utf-8')

class TestConfiguration:
    def test_init_valid(self):
        Configuration(flags_configuration=FLAGS_CONFIG)

    def test_bandit_configuration():
        # Initialize Configuration with both flags and bandits
        config = Configuration(flags_configuration=FLAGS_CONFIG, bandits_configuration=BANDITS_CONFIG)

        # Call get_bandit_keys and check the output
        bandit_keys = config.get_bandit_keys()
        assert isinstance(bandit_keys, set)
        assert bandit_keys == {"car_bandit"}

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
