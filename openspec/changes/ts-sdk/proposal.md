## Why

MVP closes when a Web client can CRUD through the Data API with Publishable Key ± App User Session. The official `@indiebase/sdk` thin wrapper over `@supabase/postgrest-js` is the product-facing half of Phase 5 (server App Auth is `app-user-session`).

## What Changes

- New package `packages/sdk-ts` (`@indiebase/sdk`): `createIndiebaseClient` per [mvp-sdk.md](../../../docs/prd/mvp-sdk.md).
- Base URL `{projectUrl}/api/data/{projectId}`; headers `X-Indiebase-Api-Key` + optional `Authorization` after `auth.setSession`.
- Do not pass PostgREST `schema` / `Accept-Profile` — gateway injects from URL.
- Integration tests against live gateway + PostgREST: select/insert/update/delete; anon vs authenticated; §6.2.3 cross-path 403 cases where client-visible.
- Sync `docs/prd/mvp-phases.md` Phase 5 SDK acceptance and `mvp-sdk.md` if DX drifts.
- Depends on `data-api-gateway`, `metadata-table-designer`, `app-user-session`.

## Capabilities

### New Capabilities

- `ts-sdk`: Official Web TypeScript Data API client; session helper; integration test matrix aligned with mvp-sdk §7.

### Modified Capabilities

- (none required in main Rust specs; document SDK in package README + PRD only unless OpenAPI examples need SDK header notes — optional `api-docs` delta for client header examples).

## Non-goals

- Manager API / Secret Key / Dashboard Session clients.
- React Native / Android / iOS SDKs (`todo.md` §5).
- Client Storage API (`todo.md` §6).
- Generated table types / GraphQL.

## Impact

- `packages/sdk-ts`: package.json, TypeScript sources, tests.
- CI / just recipes may add `pnpm`/`npm` test if not present.
- PRD Phase 5 SDK checkboxes + mvp-sdk.md.
