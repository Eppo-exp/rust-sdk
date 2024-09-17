import pytest

# In rust implementation, `eppo_client.bandit.BanditResult` is an alias
# to `eppo_client.EvaluationResult`.
from eppo_client.bandit import BanditResult


def test_bandit_result_to_string_variation():
    result = BanditResult(variation="variation", action=None)
    assert result.to_string() == "variation"


def test_bandit_result_to_string_action():
    result = BanditResult(variation="variation", action="action")
    assert result.to_string() == "action"


# Native python implementation does not convert variation to string.
@pytest.mark.rust_only
def test_bandit_result_to_string_number_variation():
    result = BanditResult(variation=13, action=None)
    assert result.to_string() == "13"
