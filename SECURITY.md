# Security Policy

Paperjam parses untrusted document formats (PDF, DOCX, XLSX, EPUB). Parser
vulnerabilities — memory safety issues, panics on crafted input, denial of
service via pathological files, or any unexpected behavior on malicious
input — are in scope and taken seriously.

## Reporting a vulnerability

**Do not open a public issue for security reports.**

Use GitHub's private vulnerability reporting:

1. Open <https://github.com/ByteVeda/paperjam/security/advisories/new>
2. Fill in a description and, if possible, a minimal reproducer
3. Attach any crafted input files to the advisory — **do not commit them
   to the repository**

## What to include

- paperjam version (`pip show paperjam` or crate version from `Cargo.toml`)
- Platform, Python version, and Rust version if building from source
- A minimal file or snippet that triggers the issue
- Observed vs. expected behavior
- Any analysis you already have (stack trace, ASan output, etc.)

## Response expectations

Paperjam is maintained by a single person, so response times are
best-effort:

- Initial acknowledgement: within 7 days
- Triage and severity assessment: within 14 days
- Fix or mitigation timeline: depends on severity and complexity

## Supported versions

Security fixes land on the latest released version only. Pin a specific
version in production and upgrade when a security release is cut.

## Scope

**In scope**

- Memory safety issues (crashes, UAF, buffer overruns) in Rust parser
  code reachable from the Python, WASM, or CLI frontends
- Panics or unhandled errors triggered by crafted input that should be
  returned as graceful errors
- Denial of service via pathological inputs (exponential blowup,
  infinite loops, quadratic behavior on small files)
- Unintended filesystem or network access from the parser

**Out of scope**

- Bugs in upstream dependencies — report those upstream; paperjam will
  update once a fix is released
- Issues that require an attacker-controlled build environment or an
  already-compromised system
- Missing hardening features that are not actual vulnerabilities
