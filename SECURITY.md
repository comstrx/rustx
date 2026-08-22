# Security Policy

We take security seriously.

- Please report vulnerabilities privately.
- Do not open public Issues for security reports.

## Report a vulnerability

Preferred (fastest):

- [GitHub Private Vulnerability Reporting / Security Advisories](https://github.com/comstrx/rustx/security/advisories/new) 🔒

If the above is not available, email the maintainer privately: <comstrx@gmail.com>

## What belongs here

✅ Security reports include:

- remote code execution, auth bypass, data exposure, privilege escalation
- supply-chain or verification issues with clear impact
- unsafe defaults that affect real deployments

❌ Not security reports (use Issues/Discussions instead):

- general bugs, feature requests, usage questions -> [Issues](https://github.com/comstrx/rustx/issues) / [Discussions](https://github.com/comstrx/rustx/discussions)
- crashes without security impact details

## Include this (makes triage fast)

- affected crate(s) + the commit or tag you are on
- impact (what can an attacker do?) + assumptions / threat model
- minimal reproduction or PoC (safe and small)
- environment details (OS/arch, `rustc --version`)
- relevant logs / error output

🚫 Do not include secrets (tokens, private keys, credentials, personal data).

## Responsible disclosure

- Please avoid public disclosure until a fix is available.
- We will coordinate on a timeline, patch, and advisory when confirmed.
- When appropriate, we disclose via releases and ecosystem advisories (e.g., RustSec).
