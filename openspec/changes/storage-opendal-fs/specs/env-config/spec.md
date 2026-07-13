## ADDED Requirements

### Requirement: Storage root configuration

The API SHALL require a non-empty storage root path (e.g. `STORAGE_ROOT`) for the OpenDAL `fs` backend, documented in `.env.example`.

#### Scenario: Missing STORAGE_ROOT fails config

- **WHEN** `STORAGE_ROOT` is unset after dotenv load
- **THEN** config loading fails with a missing `STORAGE_ROOT` error
