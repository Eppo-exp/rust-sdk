# mock-server

This is a simple mock server for use in client tests. It just serves a bunch of static files from `sdk-test-data` at the appropriate locations.

The mock server may serve multiple "environments" at the same time at different prefixes, so the base URL format is `http://localhost:8378/{env_name}/api`.

We currently have three:
- `ufc` is the default `flags-v1.json`.
- `obfuscated` is the obfuscated version of `ufc`.
- `bandits` is bandit flags and bandit models.

See `prepare.js` for the full list of files and environments served.

# Q&A

## Why port 8378?

8378 is "test" in T9.
