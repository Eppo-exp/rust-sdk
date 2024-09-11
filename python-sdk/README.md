# Eppo Python SDK

[![Test and lint SDK](https://github.com/Eppo-exp/rust-sdk/actions/workflows/python.yml/badge.svg)](https://github.com/Eppo-exp/rust-sdk/actions/workflows/python.yml)

[Eppo](https://www.geteppo.com/) is a modular flagging and experimentation analysis tool. Eppo's Python SDK is built to make assignments in multi-user server side contexts. Before proceeding you'll need an Eppo account.

## Features

- Feature gates
- Kill switches
- Progressive rollouts
- A/B/n experiments
- Mutually exclusive experiments (Layers)
- Holdouts
- Contextual multi-armed bandits
- Dynamic configuration

## Installation

```shell
pip install eppo-server-sdk
```

## Quick start

Begin by initializing a singleton instance of Eppo's client. Once initialized, the client can be used to make assignments anywhere in your app.

#### Initialize once

```python
import eppo_client
from eppo_client import Config, AssignmentLogger

client_config = Config(
    api_key="<SDK-KEY-FROM-DASHBOARD>", assignment_logger=AssignmentLogger()
)
eppo_client.init(client_config)
```


#### Assign anywhere

```python
import eppo_client

client = eppo_client.get_instance()
user = get_current_user()

variation = client.get_boolean_assignment(
    'show-new-feature',
    user.id,
    { 'country': user.country },
    False
)
```

## Assignment functions

Every Eppo flag has a return type that is set once on creation in the dashboard. Once a flag is created, assignments in code should be made using the corresponding typed function:

```python
get_boolean_assignment(...)
get_numeric_assignment(...)
get_integer_assignment(...)
get_string_assignment(...)
get_json_assignment(...)
```

Each function has the same signature, but returns the type in the function name. For booleans use `get_boolean_assignment`, which has the following signature:

```python
get_boolean_assignment(
    flag_key: str,
    subject_key: str,
    subject_attributes: Dict[str, Union[str, int, float, bool, None]],
    default_value: bool
) -> bool:
  ```

## Initialization options

The `init` function accepts the following optional configuration arguments.

| Option | Type | Description | Default |
| ------ | ----- | ----- | ----- |
| **`assignment_logger`**  | AssignmentLogger | A callback that sends each assignment to your data warehouse. Required only for experiment analysis. See [example](#assignment-logger) below. | `None` |
| **`is_graceful_mode`** | bool | When true, gracefully handles all exceptions within the assignment function and returns the default value. | `True` |
| **`poll_interval_seconds`** | int | The interval in seconds at which the SDK polls for configuration updates. | `30` |
| **`poll_jitter_seconds`** | int | The jitter in seconds to add to the poll interval. | `30` |

## Assignment logger

To use the Eppo SDK for experiments that require analysis, pass in a callback logging function to the `init` function on SDK initialization. The SDK invokes the callback to capture assignment data whenever a variation is assigned. The assignment data is needed in the warehouse to perform analysis.

The code below illustrates an example implementation of a logging callback using [Segment](https://segment.com/), but you can use any system you'd like. The only requirement is that the SDK receives a `log_assignment` callback function. Here we define an implementation of the Eppo `SegmentAssignmentLogger` interface containing a single function named `log_assignment`:

```python
from eppo_client import AssignmentLogger, Config
import analytics

# Connect to Segment.
analytics.write_key = "<SEGMENT_WRITE_KEY>"

class SegmentAssignmentLogger(AssignmentLogger):
    def log_assignment(self, assignment):
        analytics.track(assignment["subject"], "Eppo Randomization Assignment", assignment)

client_config = Config(api_key="<SDK-KEY-FROM-DASHBOARD>", assignment_logger=SegmentAssignmentLogger())
```

## Export configuration

To support the use-case of needing to bootstrap a front-end client, the Eppo SDK provides a function to export flag configurations to a JSON string.

Use the `Configuration.get_flags_configuration` function to export flag configurations to a JSON string and then send it to the front-end client.

```python
from fastapi import JSONResponse

import eppo_client
import json

client = eppo_client.get_instance()
flags_configuration = client.get_configuration().get_flags_configuration()

# Create a JSONResponse object with the stringified JSON
response = JSONResponse(content={"flagsConfiguration": flags_configuration})
```

## Philosophy

Eppo's SDKs are built for simplicity, speed and reliability. Flag configurations are compressed and distributed over a global CDN (Fastly), typically reaching your servers in under 15ms. Server SDKs continue polling Eppo’s API at 30-second intervals. Configurations are then cached locally, ensuring that each assignment is made instantly. Evaluation logic within each SDK consists of a few lines of simple numeric and string comparisons. The typed functions listed above are all developers need to understand, abstracting away the complexity of the Eppo's underlying (and expanding) feature set.

## Contributing

To publish a new version of the SDK, set the version as desired in `eppo_client/version.py`, then create a new Github release. The CI/CD configuration will handle the build and publish to PyPi.
