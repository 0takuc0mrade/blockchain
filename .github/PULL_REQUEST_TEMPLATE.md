## Summary
<!-- Brief description of the changes and why they were made. Link to related issue if applicable. -->

## Changes
<!-- List the main changes in this PR. -->

- 

## Testing
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `stellar contract build` succeeds
- [ ] New tests added for new functionality
- [ ] Existing tests still pass (no regressions)

## Security review
<!-- For contract changes, confirm these have been considered. -->

- [ ] `require_auth()` called before any storage read or business logic (issue #206)
- [ ] State writes ordered before external calls (token transfers)
- [ ] No new admin/organizer/owner capabilities without explicit justification
- [ ] New persistent entries have `extend_ttl` policy documented in STORAGE_TTL.md
- [ ] New error codes added to ERRORS.md with retry semantics
- [ ] New events emitted on all state-changing paths
- [ ] No new upgrade/redeploy requirements introduced

## Documentation
- [ ] ARCHITECTURE.md updated (if storage/auth/flows changed)
- [ ] THREAT_MODEL.md updated (if trust boundaries/attack surface changed)
- [ ] CONTRACT_API.md updated (if public interface changed)
- [ ] INTEGRATION.md updated (if backend integration affected)
- [ ] CAP_ROYALTY_ROUNDING.md updated (if formulas changed)
- [ ] CHANGELOG.md updated

## Deployment
<!-- If this requires a new deployment, fill out the deployment checklist. -->

- [ ] Not a contract change / no deployment needed
- [ ] Deployment checklist completed (see DEPLOYMENT.md)
- [ ] DEPLOYMENTS.md updated with new deployment record