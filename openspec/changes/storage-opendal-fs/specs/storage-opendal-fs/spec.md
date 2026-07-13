## ADDED Requirements

### Requirement: OpenDAL fs storage root

The API SHALL store project files via OpenDAL `fs` under a configured root directory, with object keys namespaced by `project_id`.

#### Scenario: Upload writes under project prefix

- **WHEN** an authorized caller uploads a file to a project
- **THEN** the object is stored under `{project_id}/…` within the configured storage root

### Requirement: Manager file routes

The API SHALL expose `GET/POST /api/projects/{project_id}/files` and `GET/DELETE /api/projects/{project_id}/files/{file_id}` for list, multipart upload, download, and delete.

#### Scenario: Owner uploads and lists

- **WHEN** project owner uploads a file then lists files
- **THEN** the new file appears in the list with a stable `file_id`

#### Scenario: Download returns bytes

- **WHEN** an authorized caller downloads by `file_id`
- **THEN** response is 200 with the original content type and body

### Requirement: Project role matrix for Storage

With Dashboard Session, `owner` and `admin` MAY upload/list/download/delete. `member` MAY list/download and MUST be denied upload/delete (`403`). Publishable Key MUST NOT authorize Storage routes.

#### Scenario: Member write denied

- **WHEN** a `member` attempts `POST` or `DELETE` on Storage routes
- **THEN** response is 403

#### Scenario: Publishable Key rejected

- **WHEN** Storage is called with only a Publishable Key
- **THEN** response is 401 or 403

### Requirement: Secret Key Storage access

The API SHALL allow Secret Key (bound to the URL project) full file CRUD on Storage routes and SHALL emit an audit log for Secret Key Storage mutations.

#### Scenario: Secret Key upload

- **WHEN** a valid Secret Key uploads to `/api/projects/{project_id}/files`
- **THEN** upload succeeds and an audit event is recorded
