from typing import Dict, Any, Set, Union

__version__: str

def init(config: ClientConfig) -> EppoClient: ...
def get_instance() -> EppoClient: ...

class Configuration:
    def __init__(
        self, *, flags_configuration: bytes, bandits_configuration: bytes | None = None
    ) -> None: ...
    def get_flags_configuration(self) -> bytes: ...
    def get_bandits_configuration(self) -> bytes | None: ...
    def get_flag_keys(self) -> Set[str]: ...
    def get_bandit_keys(self) -> Set[str]: ...

class ClientConfig:
    api_key: str
    base_url: str
    assignment_logger: AssignmentLogger
    is_graceful_mode: bool
    poll_interval_seconds: int | None
    poll_jitter_seconds: int
    initial_configuration: Configuration | None

    def __init__(
        self,
        *,
        api_key: str,
        base_url: str = ...,
        assignment_logger: AssignmentLogger,
        is_graceful_mode: bool = True,
        poll_interval_seconds: int | None = ...,
        poll_jitter_seconds: int = ...,
        initial_configuration: Configuration | None = None
    ): ...

class AssignmentLogger:
    def log_assignment(self, event: Dict) -> None: ...
    def log_bandit_action(self, event: Dict) -> None: ...

class EppoClient:
    def get_string_assignment(
        self,
        flag_key: str,
        subject_key: str,
        subject_attributes: Dict[str, Union[str, int, float, bool, None]],
        default: str,
    ) -> str: ...
    def get_integer_assignment(
        self,
        flag_key: str,
        subject_key: str,
        subject_attributes: Dict[str, Union[str, int, float, bool, None]],
        default: int,
    ) -> int: ...
    def get_numeric_assignment(
        self,
        flag_key: str,
        subject_key: str,
        subject_attributes: Dict[str, Union[str, int, float, bool, None]],
        default: float,
    ) -> float: ...
    def get_boolean_assignment(
        self,
        flag_key: str,
        subject_key: str,
        subject_attributes: Dict[str, Union[str, int, float, bool, None]],
        default: bool,
    ) -> bool: ...
    def get_json_assignment(
        self,
        flag_key: str,
        subject_key: str,
        subject_attributes: Dict[str, Union[str, int, float, bool, None]],
        default: Any,
    ) -> Any: ...
    def get_string_assignment_details(
        self,
        flag_key: str,
        subject_key: str,
        subject_attributes: Dict[str, Union[str, int, float, bool, None]],
        default: str,
    ) -> EvaluationResult: ...
    def get_integer_assignment_details(
        self,
        flag_key: str,
        subject_key: str,
        subject_attributes: Dict[str, Union[str, int, float, bool, None]],
        default: int,
    ) -> EvaluationResult: ...
    def get_numeric_assignment_details(
        self,
        flag_key: str,
        subject_key: str,
        subject_attributes: Dict[str, Union[str, int, float, bool, None]],
        default: float,
    ) -> EvaluationResult: ...
    def get_boolean_assignment_details(
        self,
        flag_key: str,
        subject_key: str,
        subject_attributes: Dict[str, Union[str, int, float, bool, None]],
        default: bool,
    ) -> EvaluationResult: ...
    def get_json_assignment_details(
        self,
        flag_key: str,
        subject_key: str,
        subject_attributes: Dict[str, Union[str, int, float, bool, None]],
        default: Any,
    ) -> EvaluationResult: ...
    def get_bandit_action(
        self,
        flag_key: str,
        subject_key: str,
        subject_context: (
            ContextAttributes | Dict[str, Union[str, int, float, bool, None]]
        ),
        actions: (
            Dict[str, ContextAttributes]
            | Dict[str, Dict[str, Union[str, int, float, bool, None]]]
        ),
        default: str,
    ) -> EvaluationResult: ...
    def get_bandit_action_details(
        self,
        flag_key: str,
        subject_key: str,
        subject_context: (
            ContextAttributes | Dict[str, Union[str, int, float, bool, None]]
        ),
        actions: (
            Dict[str, ContextAttributes]
            | Dict[str, Dict[str, Union[str, int, float, bool, None]]]
        ),
        default: str,
    ) -> EvaluationResult: ...
    def get_configuration(self) -> Configuration | None: ...
    def set_configuration(self, configuration: Configuration): ...
    def get_flag_keys(self) -> Set[str]: ...
    def get_bandit_keys(self) -> Set[str]: ...
    def set_is_graceful_mode(self, is_graceful_mode: bool): ...
    def is_initialized(self) -> bool: ...
    def wait_for_initialization(self) -> None: ...

class ContextAttributes:
    def __new__(
        cls,
        numeric_attributes: Dict[str, float],
        categorical_attributes: Dict[str, str],
    ): ...
    @staticmethod
    def empty() -> ContextAttributes: ...
    @staticmethod
    def from_dict(
        attributes: Dict[str, Union[str, int, float, bool, None]]
    ) -> ContextAttributes: ...
    @property
    def numeric_attributes(self) -> Dict[str, float]: ...
    @property
    def categorical_attributes(self) -> Dict[str, str]: ...

class EvaluationResult:
    variation: Any
    action: str | None
    evaluation_details: Any | None
    def __new__(
        cls,
        variation: Any,
        action: str | None = None,
        evaluation_details: Any | None = None,
    ): ...
    def to_string(self) -> str: ...
