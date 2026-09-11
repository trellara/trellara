## Summary

<!-- What can an operator, user, or contributor do now that they could not do before? -->

## Why

<!-- Link the issue or explain the concrete failure, friction, or evidence gap. -->

## Correctness impact

<!-- Name affected boundaries: capture, transaction assembly, durable publication, source ACK,
apply/checkpoint, quarantine, DDL, lake completeness, verification, or none. Explain fail-closed
behavior and any compatibility or migration impact. -->

## Verification

<!-- List exact commands and results. Identify service-backed checks that were not run. -->

- [ ] Narrow crate tests pass
- [ ] `make ci` passes
- [ ] Relevant service-backed tests pass, or the omission is explained
- [ ] `git diff --check` passes

## Documentation and evidence

- [ ] User-facing documentation is updated, or no update is needed
- [ ] Owning crate `README.md` and `skills.md` are updated, or no update is needed
- [ ] Generated correctness evidence is refreshed, or no output changed
- [ ] Logs, configuration, and fixtures contain no secrets or customer data
