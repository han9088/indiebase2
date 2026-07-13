## 1. Package scaffold

- [ ] 1.1 Create `packages/sdk-ts` with package.json, TypeScript config, dependency on `@supabase/postgrest-js`
- [ ] 1.2 Implement `createIndiebaseClient` + `auth.setSession` / `auth.signOut` per mvp-sdk.md
- [ ] 1.3 Export public API from `src/index.ts`; add package README

## 2. Integration tests

- [ ] 2.1 Add E2E tests for select/insert/update/delete (anon + authenticated) against local API
- [ ] 2.2 Cover/document §6.2.3 cross-path 403 expectations relevant to SDK clients
- [ ] 2.3 Wire `just` / CI recipe to run SDK tests when stack is up (or document skip flag)

## 3. PRD

- [ ] 3.1 Sync `docs/prd/mvp-phases.md` Phase 5 SDK acceptance and `mvp-sdk.md` if DX differs
- [ ] 3.2 Verify package builds (`tsc` / test runner green)
