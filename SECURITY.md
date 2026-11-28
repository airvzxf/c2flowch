# Security policy

## Supported versions

Only the latest release gets security fixes. Update to it before you
report a problem.

## Reporting a vulnerability

Report it privately with
[GitHub's private vulnerability reporting](https://github.com/airvzxf/c2flowch/security/advisories/new),
not in a public issue. Include the c2flowch version, the input (a C
file or a YAML IR file) and the command that shows the problem.

When the fix is released, the advisory is published and the CHANGELOG
names it under "Security".

## Checking a download

Each release archive has a `.sha256` file and a build provenance
attestation. `gh attestation verify <archive> -R airvzxf/c2flowch` checks
that the release workflow of this repository built it (from version
2.5.4 on). See "Install" in the [README](README.md).

## What the project already checks

- `cargo audit` runs on every change and once a week against the
  RustSec advisory database (`.github/workflows/audit.yml`).
- CodeQL scans the Rust sources and the workflows
  (`.github/workflows/codeql.yml`).
- Dependabot updates the dependencies and the pinned GitHub Actions.
