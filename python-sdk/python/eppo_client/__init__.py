from typing import Any, Dict, Set, Union, Type

# Rust currently does not define submodules as packages, so Rust
# submodules are not importable from Python.[1] There is a hacky way
# to make submodules re-exportable (by tweaking sys.modules) but it
# has some drawbacks.
#
# It's more straightforward to add a bit of Python to re-export items
# at different locations.
#
# This __init__.py just re-exports everything from the Rust module.[2]
#
# [1]: https://github.com/PyO3/pyo3/issues/759
# [2]: https://www.maturin.rs/project_layout#pure-rust-project
import eppo_client._eppo_client as _eppo_client
from eppo_client._eppo_client import *
from eppo_client._eppo_client import __version__

# re-exports
from eppo_client.assignment_logger import AssignmentCacheLogger
from eppo_client.bandit import BanditResult

Attribute = Union[str, int, float, bool, None]
Attributes = Dict[str, Attribute]

__doc__ = _eppo_client.__doc__
__all__ = ["AssignmentCacheLogger", "BanditResult", "Attribute", "Attributes"]
if hasattr(_eppo_client, "__all__"):
    __all__.extend(_eppo_client.__all__)
