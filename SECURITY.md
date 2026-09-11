# Security policy

Trellara moves database changes and produces correctness evidence. A vulnerability can expose
credentials, weaken a durability boundary, or make an unsafe state look verified. Please report
suspected vulnerabilities privately.

## Supported versions

Trellara is pre-1.0 and does not yet publish a production-supported release line.

| Version | Security fixes |
| --- | --- |
| Current `main` | Yes, during pre-release development |
| Most recent tagged release | Yes, once releases exist |
| Older commits or tags | No |

## Report a vulnerability

Use GitHub's private **Report a vulnerability** form:

<https://github.com/trellara/trellara/security/advisories/new>

Do not open a public issue, pull request, or discussion for an undisclosed vulnerability. Do not
include live credentials, customer data, or unrestricted database rows in a report. If a secret was
exposed, revoke or rotate it before sending evidence.

A useful report includes:

- the affected commit, version, crate, command, or deployment mode;
- impact and the trust boundary that fails;
- reproducible steps or a minimal proof of concept;
- required configuration and PostgreSQL, broker, catalog, or operating-system versions;
- whether exploitation requires authenticated or privileged access; and
- a suggested fix or mitigation, if known.

## What belongs here

Please report privately when the issue could cause:

- credential, token, connection-string, row-data, or evidence disclosure;
- authentication or authorization bypass;
- remote code execution, command injection, or unsafe file access;
- forged, incomplete, or conflicting data being accepted as durably committed or verified;
- source acknowledgement before the configured durable boundary;
- bypass of target deduplication, checkpoint atomicity, quarantine, or DDL barriers;
- tampering with release artifacts, generated evidence, or the CI supply chain; or
- denial of service from a small, attacker-controlled input.

Ordinary bugs, documented limitations, performance questions, and non-sensitive correctness issues
can use the public issue tracker. When uncertain, report privately first.

## Response process

Maintainers aim to acknowledge a complete report within three business days, confirm impact and a
remediation plan as soon as practical, and coordinate disclosure after a fix is available. Timing
depends on severity, reproducibility, downstream coordination, and release readiness.

We will credit reporters who request credit. Please allow maintainers a reasonable remediation
window before public disclosure.
