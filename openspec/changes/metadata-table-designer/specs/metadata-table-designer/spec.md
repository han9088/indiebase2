## ADDED Requirements

### Requirement: Metadata tables

The system SHALL persist `public.table_metadata` and `public.column_metadata` (soft-deletable) including per-table `allow_anon_read` (default false). Metadata MUST reference `project_id` and the physical table/column names in `proj_{ulid}`.

#### Scenario: Create table writes metadata

- **WHEN** an authorized member creates a table via Manager API
- **THEN** a `table_metadata` row and corresponding `column_metadata` rows exist for that project

### Requirement: Manager table and column DDL

The API SHALL provide Manager endpoints (Dashboard Session + project membership) to create, update, and delete tables and columns. DDL MUST run in schema `proj_{ulid}`. Clients MUST NOT supply arbitrary SQL; only validated identifiers and whitelisted types.

#### Scenario: Owner creates users table

- **WHEN** project owner creates table `users` with columns via Manager API
- **THEN** relation `proj_{ulid}.users` exists and matches metadata

#### Scenario: Member cannot DDL

- **WHEN** a `member` attempts to create or drop a table
- **THEN** response is 403

### Requirement: Bootstrap RLS on create

On table create, the system SHALL enable RLS and install MVP bootstrap policies: anon default deny (SELECT only if `allow_anon_read`); authenticated default allow CRUD; operator/service behavior via DB roles as in architecture §11.3.

#### Scenario: Anon denied by default

- **WHEN** Publishable Key without App User Session reads a new table with `allow_anon_read=false`
- **THEN** PostgREST/RLS denies the read

#### Scenario: allow_anon_read enables SELECT only

- **WHEN** `allow_anon_read` is set true for that table
- **THEN** anon MAY SELECT and MUST NOT INSERT/UPDATE/DELETE via RLS

### Requirement: PostgREST schema reload after DDL

After successful DDL, the system SHALL trigger PostgREST schema reload so new tables are immediately queryable via Data API.

#### Scenario: New table visible to Data API

- **WHEN** a table is created and reload completes
- **THEN** Data API requests for that table are accepted by PostgREST (not schema-cache miss)
