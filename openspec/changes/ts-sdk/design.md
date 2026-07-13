## Context

Server Data API + App Auth (when landed) enable Web clients. mvp-sdk.md specifies a thin `@supabase/postgrest-js` wrapper. No `packages/` SDK exists yet.

## Goals / Non-Goals

**Goals:**

- Ship `packages/sdk-ts` with `createIndiebaseClient`.
- Session helpers `auth.setSession` / `auth.signOut`.
- Integration tests against running API (compose) for CRUD + 403 matrix where applicable.

**Non-Goals:** Manager SDK, Secret Key helpers, multi-platform SDKs, Storage client.

## Decisions

### 1. Package manager

- **Decision:** Add minimal `package.json` + TypeScript; use `pnpm` or `npm` consistent with any existing monorepo tooling — if none, npm workspaces at repo root is fine for one package.
- **Alternatives:** Publish-only without workspace — still keep sources in-repo.

### 2. Dependency

- **Decision:** Depend on `@supabase/postgrest-js` only (not full supabase-js).

### 3. Auth surface

- **Decision:** Client-side session holder only; optional thin `signIn` that calls `/api/auth/app/login` can be included if App Auth is ready — otherwise document manual `setSession` first (mvp-sdk sketch).

### 4. Tests

- **Decision:** Vitest or node:test hitting live `projectUrl`; skip when `INDIEBASE_E2E=0`.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Server not ready | Gate package merge until gateway+auth; or ship client with deferred E2E |
| postgrest-js version drift | Pin major; test Prefer/Range |

## Migration Plan

1. Scaffold package.
2. Implement client.
3. E2E against local stack; update PRD Phase 5.

## Open Questions

- Publish name `@indiebase/sdk` vs private scope — use `@indiebase/sdk` as in PRD.
