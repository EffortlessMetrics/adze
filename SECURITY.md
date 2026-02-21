# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| main    | Yes       |

This project has not yet published stable releases to crates.io. All code on
`main` should be considered pre-release. We still take security seriously and
will address reported issues promptly.

## Reporting a Vulnerability

**Please do not open a public issue for security vulnerabilities.**

Instead, report through one of the following channels:

1. **GitHub Security Advisories** (preferred):
   [Report a vulnerability](https://github.com/EffortlessMetrics/adze/security/advisories/new)

2. **Email**: git@effortlesssteven.com

Please include:
- A description of the vulnerability
- Steps to reproduce the issue
- Any relevant logs or error output
- The severity you believe this represents

## Response Timeline

- **Acknowledgment**: Within 72 hours
- **Initial assessment**: Within one week
- **Fix or mitigation**: Depends on severity, but we aim for patches within 30 days for critical issues

## Scope

The following are in scope for security reports:

- Memory safety issues in parsing (buffer overflows, use-after-free)
- Denial of service through crafted grammars or input
- Unsafe FFI boundary issues
- Build-time code execution vulnerabilities

## Disclosure

We follow coordinated disclosure. We will work with you on a timeline for
public disclosure and credit you in the advisory (unless you prefer otherwise).
