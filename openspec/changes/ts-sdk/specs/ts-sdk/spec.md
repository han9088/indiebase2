## ADDED Requirements

### Requirement: createIndiebaseClient factory

The package `@indiebase/sdk` SHALL export `createIndiebaseClient({ projectUrl, projectId, publishableKey })` that builds a PostgREST client base URL `{projectUrl}/api/data/{projectId}` and sends `X-Indiebase-Api-Key` on every request. Callers MUST NOT be required to set `Accept-Profile` / schema options.

#### Scenario: Select with Publishable Key

- **WHEN** client calls `.from(table).select()` against a live gateway with only Publishable Key
- **THEN** the request reaches Data API with the Publishable Key header

### Requirement: Session helpers

The client SHALL expose `auth.setSession({ accessToken })` and `auth.signOut()` that add or remove `Authorization: Bearer` on subsequent requests without changing the Publishable Key header.

#### Scenario: Authenticated requests after setSession

- **WHEN** `setSession` is called with an App User Session token
- **THEN** subsequent requests include both `X-Indiebase-Api-Key` and `Authorization: Bearer`

#### Scenario: signOut clears Authorization

- **WHEN** `signOut` is called
- **THEN** subsequent requests omit Authorization while keeping the Publishable Key

### Requirement: Integration matrix

The SDK package SHALL include integration tests (runnable against local compose) covering select/insert/update/delete for anon and authenticated modes where server RLS allows, and documenting expected 403 cross-path cases per mvp-sdk §7.

#### Scenario: CRUD integration passes

- **WHEN** E2E tests run against a prepared project with bootstrap RLS tables
- **THEN** authenticated CRUD cases pass and anon cases match allow_anon_read policy
