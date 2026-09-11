# Claude Code instructions

The canonical repository instructions are in [AGENTS.md](AGENTS.md). Read and follow that file
before making changes.

For work inside a crate, also read that crate's `README.md` and `skills.md`. Preserve Trellara's
durability, acknowledgement, atomic-apply, fail-closed recovery, and evidence contracts; run the
narrow crate checks while iterating and `make ci` before handing off a repository-wide change.

Do not assume personal or machine-local skills, plugins, aliases, or commands are available to other
contributors. Keep durable project guidance in this repository and keep personal tool routing in
user-level configuration.
