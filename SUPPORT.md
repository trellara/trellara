# Support

Trellara is pre-1.0 software under active development. Community support is best-effort; there is no
production support SLA yet.

## Ask a question

Search the [documentation](README.md#documentation) and existing issues first. If the answer is not
there, open an issue with the `question` label and include:

- the commit or version you are using;
- PostgreSQL, operating-system, broker, or catalog versions involved;
- the command and intended outcome;
- sanitized configuration; and
- the smallest logs or evidence that explain the problem.

Never post passwords, connection strings, tokens, certificates, customer data, or unrestricted row
contents. Use `trellara config redact` before sharing configuration.

## Report a bug

Use the bug-report issue form. Include a minimal reproduction and say which checks or service-backed
tests you ran. Correctness failures should identify the last trustworthy boundary: source capture,
durable publication, source acknowledgement, target apply, checkpoint, or verification.

## Propose a feature

Use the feature-request issue form. Lead with the operator problem and the evidence that would prove
the feature safe, rather than only describing an API or implementation.

## Security

Do not report vulnerabilities publicly. Follow [SECURITY.md](SECURITY.md) and use GitHub's private
vulnerability-reporting form.
