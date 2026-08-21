# Support

Welcome! This page is the fastest way to get help with `rustx` with the least back-and-forth.

## Choose the right place

| You want to...                    | Go to...                                                                |
| --------------------------------- | ----------------------------------------------------------------------- |
| Ask a question / discuss design   | 💬 [Discussions](https://github.com/comstrx/rustx/discussions)          |
| Report a reproducible bug         | 🐞 [Issues](https://github.com/comstrx/rustx/issues)                    |
| Report a security issue (private) | 🔒 [Security](https://github.com/comstrx/rustx/security/advisories/new) |
| Read docs / guides                | 📚 [Docs](https://github.com/comstrx/rustx/tree/main/docs)              |
| Review recent changes             | 🧾 [Releases](https://github.com/comstrx/rustx/releases)                |

---

## Before you post (2 minutes)

- Search existing Issues/Discussions (duplicates slow everyone down).
- Confirm you are on the latest release (many bugs are already fixed).
- Reduce to a minimal reproduction (small snippet or a tiny public repo).

If you cannot reproduce it reliably, we probably cannot fix it reliably.

---

## Bug reports that get solved fast

When opening an Issue, please include:

- What happened vs what you expected
- Minimal repro: exact steps + code (or a small public repo)
- Versions:
  - `rustx` version(s)
  - `rustc --version`
- Environment: OS + architecture (Linux x86_64, macOS arm64, Windows x86_64, etc.)
- Output: exact error/logs (copy as text, not screenshots)
- If relevant: feature flags and a backtrace

Helpful links (if available):

- [Bug report template](https://github.com/comstrx/rustx/issues/new?template=bug_report.md)
- [Feature request template](https://github.com/comstrx/rustx/issues/new?template=feature_request.md)

---

## Performance reports (we love them)

If you suspect a performance regression, please add:

- a benchmark or minimal test case
- input sizes / dataset shape
- before vs after numbers
- profiling output if you have it (even a short summary)

---

## Scope and expectations

We are happy to help with:

- ✅ Reproducible bugs and regressions
- ✅ Documentation gaps and examples
- ✅ Clarifying intended behavior / API usage
- ✅ Performance regressions (with a benchmark)

We usually cannot help with:

- ❌ "It is broken" with no repro, versions, or logs
- ❌ Debugging heavily modified forks (please reproduce on upstream)
- ❌ Security reports via public Issues (use the Security Policy link)

`rustx` is community-driven. There is no guaranteed SLA, but clear reports get the fastest response.

Thanks for helping keep the project sharp. 🧠
