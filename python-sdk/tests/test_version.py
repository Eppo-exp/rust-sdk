import pytest

import eppo_client


@pytest.mark.rust_only
def test_version_available():
    assert isinstance(eppo_client.__version__, str)


@pytest.mark.rust_only
def test_min_version():
    assert eppo_client.__version__ >= "4.0.0"
