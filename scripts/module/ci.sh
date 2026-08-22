#!/usr/bin/env bash

ENSURE_TOOLS=1

ci_step () {

    local label="${1-}" cmd="${2-}"
    shift 2 || true

    [[ -n "${label}" && -n "${cmd}" ]] || die "ci_step: usage: ci_step <label> <cmd> [tool...] -- [arg...]" 2

    local -a tools=()

    while [[ $# -gt 0 && "${1}" != "--" ]]; do
        tools+=( "${1}" )
        shift || true
    done

    [[ "${1-}" == "--" ]] && shift

    (( ENSURE_TOOLS )) && (( ${#tools[@]} )) && cmd_ensure "${tools[@]}"

    info_ln "${label} ...\n"
    "${cmd}" "$@"

    success_ln "CI ${label} Succeeded.\n"

}
cmd_ci_help () {

    info_ln "CI :\n"

    printf '    %s\n' \
        "ci-stable                  * CI stable (check + test) no-default-features + all-features + release" \
        "ci-nightly                 * CI nightly (check + test) no-default-features + all-features + release" \
        "ci-msrv                    * CI msrv (check + test) no-default-features + all-features + release" \
        "" \
        "ci-doc                     * CI docs (doc-check + doc-test)" \
        "ci-panic                   * CI panic=abort (nightly + all-features)" \
        "" \
        "ci-fmt                     * CI format (fmt-check)" \
        "ci-lint                    * CI lint (conform + taplo + prettier + typos)" \
        "ci-clippy                  * CI clippy (cargo-clippy)" \
        "" \
        "ci-audit                   * CI audit (cargo-audit/deny)" \
        "ci-security                * CI security (gitleaks + trivy + sbom)" \
        "ci-vet                     * CI vet (cargo-vet)" \
        "ci-hack                    * CI hack (cargo-hack)" \
        "ci-udeps                   * CI udeps (cargo-udeps)" \
        "ci-bloat                   * CI bloat (cargo-bloat)" \
        "" \
        "ci-fuzz                    * CI fuzz (runs targets with timeout & corpus)" \
        "ci-sanitizer               * CI sanitizer detect UB" \
        "ci-miri                    * CI miri detect UB / unsafe issues" \
        "" \
        "ci-semver                  * CI Semver (check semver)" \
        "ci-coverage                * CI coverage (llvm-cov)" \
        "" \
        "ci-publish                 * CI publish gate then publish (full checks + publish)" \
        "" \
        "ci-local                   * Run a pipeline simulation ( full previous ci-xxx features )" \
        ''

}

cmd_ci_stable () {

    (( ENSURE_TOOLS )) && cmd_ensure nextest

    info_ln "Check Stable ...\n"

    cmd_check "$@"
    cmd_check --no-default-features "$@"
    cmd_check --all-features "$@"
    cmd_check --release "$@"

    info_ln "Test Stable ...\n"

    cmd_test "$@"
    cmd_test --no-default-features "$@"
    cmd_test --all-features "$@"
    cmd_test --release "$@"

    success_ln "CI Stable Succeeded.\n"

}
cmd_ci_nightly () {

    (( ENSURE_TOOLS )) && cmd_ensure nextest

    info_ln "Check Nightly ...\n"

    cmd_check --nightly "$@"
    cmd_check --nightly --no-default-features "$@"
    cmd_check --nightly --all-features "$@"
    cmd_check --nightly --release "$@"

    info_ln "Test Nightly ...\n"

    cmd_test --nightly "$@"
    cmd_test --nightly --no-default-features "$@"
    cmd_test --nightly --all-features "$@"
    cmd_test --nightly --release "$@"

    success_ln "CI Nightly Succeeded.\n"

}
cmd_ci_msrv () {

    (( ENSURE_TOOLS )) && cmd_ensure nextest

    info_ln "Check Msrv ...\n"

    cmd_check --msrv "$@"
    cmd_check --msrv --no-default-features "$@"
    cmd_check --msrv --all-features "$@"
    cmd_check --msrv --release "$@"

    info_ln "Test Msrv ...\n"

    cmd_test --msrv "$@"
    cmd_test --msrv --no-default-features "$@"
    cmd_test --msrv --all-features "$@"
    cmd_test --msrv --release "$@"

    success_ln "CI Msrv Succeeded.\n"

}

cmd_ci_doc () {

    (( ENSURE_TOOLS )) && cmd_ensure nextest

    info_ln "Check Doc ...\n"
    cmd_doc_check "$@"

    info_ln "Test Doc ...\n"
    cmd_doc_test "$@"

    success_ln "CI Doc Succeeded.\n"

}
cmd_ci_panic () {

    RUSTFLAGS="${RUSTFLAGS:-} -C panic=abort -Zpanic-abort-tests" \
        ci_step Panic cmd_test nextest -- --nightly --all-features "$@"

}

cmd_ci_fmt () {

    ci_step Format cmd_fmt_check fmt -- "$@"

}
cmd_ci_lint () {

    (( ENSURE_TOOLS )) && cmd_ensure taplo typos

    info_ln "Conform ...\n"
    cmd_conform

    info_ln "Taplo ...\n"
    cmd_taplo_check "$@"

    info_ln "Prettier ...\n"
    cmd_prettier_check "$@"

    info_ln "Typos ...\n"
    cmd_typos_check "$@"

    success_ln "CI Lint Succeeded.\n"

}
cmd_ci_clippy () {

    ci_step Clippy cmd_clippy clippy -- "$@"

}

cmd_ci_audit () {

    ci_step Audit cmd_audit_check audit deny -- "$@"

}
cmd_ci_security () {

    (( ENSURE_TOOLS )) && cmd_ensure gitleaks trivy syft

    info_ln "Gitleaks ...\n"
    cmd_leaks "$@"

    info_ln "Trivy ...\n"
    cmd_trivy "$@"

    info_ln "SBOM ...\n"
    cmd_sbom "$@"

    success_ln "CI Security Succeeded.\n"

}
cmd_ci_vet () {

    ci_step Vet cmd_vet_check vet -- "$@"

}
cmd_ci_hack () {

    ci_step Hack cmd_hack hack -- "$@"

}
cmd_ci_udeps () {

    ci_step Udeps cmd_udeps udeps -- "$@"

}
cmd_ci_bloat () {

    ci_step Bloat cmd_bloat bloat -- "$@"

}

cmd_ci_sanitizer () {

    (( ENSURE_TOOLS )) && cmd_ensure sanitizer

    info_ln "Sanitizer ...\n"

    cmd_sanitizer asan "$@"
    cmd_sanitizer tsan "$@"
    cmd_sanitizer lsan "$@"
    cmd_sanitizer msan "$@"

    success_ln "CI Sanitizer Succeeded.\n"

}
cmd_ci_fuzz () {

    if [[ ! -d "${ROOT_DIR:-.}/fuzz" ]]; then

        warn "fuzz: no fuzz/ directory - nothing to fuzz. Run 'cargo fuzz init' to create targets."
        return 0

    fi

    ci_step Fuzz cmd_fuzz fuzz -- "$@"

}
cmd_ci_miri () {

    ci_step Miri cmd_miri miri -- "$@"

}

cmd_ci_semver () {

    ci_step Semver cmd_semver semver -- "$@"

}
cmd_ci_coverage () {

    ci_step Coverage cmd_coverage cov -- --upload "$@"

}
cmd_ci_publish () {

    ci_step Publish cmd_publish -- "$@"

}

cmd_ci_local () {

    ENSURE_TOOLS=0
    cmd_ensure

    cmd_ci_stable
    cmd_ci_nightly
    cmd_ci_msrv

    cmd_ci_doc
    cmd_ci_panic

    cmd_ci_fmt
    cmd_ci_lint
    cmd_ci_clippy

    cmd_ci_audit
    cmd_ci_security
    cmd_ci_vet
    cmd_ci_hack
    cmd_ci_udeps
    cmd_ci_bloat

    cmd_ci_fuzz
    cmd_ci_sanitizer
    cmd_ci_miri
    cmd_ci_semver

    cmd_ci_coverage --no-upload
    cmd_ci_publish --dry-run

    success_ln "CI Pipeline Succeeded.\n"

}
