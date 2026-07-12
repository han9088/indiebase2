## ADDED Requirements

### Requirement: Server establishes Postgres and Redis clients at startup

The server SHALL create a Postgres pool (sqlx) and a Redis client from `Config` during startup. Failure to connect MUST fail startup with a clear error (non-zero exit). `GET /health` MAY remain liveness-only without probing dependencies.

#### Scenario: Startup with reachable infra

- **WHEN** Postgres and Redis are reachable with configured credentials
- **THEN** the server starts and holds usable pool/client handles for Manager routes

#### Scenario: Startup fails when Postgres unreachable

- **WHEN** Postgres cannot be connected at startup
- **THEN** the process exits with a non-zero status and an error mentioning the database
