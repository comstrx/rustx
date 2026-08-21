#!/usr/bin/env bash

ENSURE_TOOLS=1

cmd_ci_help () {

    info_ln "CI :\n"

    printf '    %s\n' \
        "ci-stable                  * CI stable (check + test) no-default-features + all-features + release" \
        "ci-nightly                 * CI nightly (check + test) no-default-features + all-features + release" \
        "ci-msrv                    * CI msrv (check + test) no-default-features + all-features + release" \
        "" \
        "ci-doc                     * CI docs (doc-check + doc-test)" \
        "ci-bench                   * CI benches (check --benches)" \
        "ci-example                 * CI examples (check --examples)" \
        "ci-panic                   * CI panic=abort (nightly + all-features)" \
        "" \
        "ci-fmt                     * CI format (fmt-check)" \
        "ci-lint                    * CI lint (taplo + prettier + typos)" \
        "ci-clippy                  * CI clippy (cargo-clippy)" \
        "" \
        "ci-audit                   * CI audit (cargo-audit/deny)" \
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
cmd_ci_bench () {

    info_ln "Check Benches ...\n"
    cmd_check --benches "$@"

    success_ln "CI Bench Succeeded.\n"

}
cmd_ci_example () {

    info_ln "Check Examples ...\n"
    cmd_check --examples "$@"

    success_ln "CI Example Succeeded.\n"

}
cmd_ci_panic () {

    (( ENSURE_TOOLS )) && cmd_ensure nextest

    info_ln "Panic ...\n"
    RUSTFLAGS="${RUSTFLAGS:-} -C panic=abort -Zpanic-abort-tests" cmd_test --nightly --all-features "$@"

    success_ln "CI Panic Succeeded.\n"

}

cmd_ci_fmt () {

    (( ENSURE_TOOLS )) && cmd_ensure fmt

    info_ln "Format ...\n"
    cmd_fmt_check "$@"

    success_ln "CI Format Succeeded.\n"

}
cmd_ci_lint () {

    (( ENSURE_TOOLS )) && cmd_ensure taplo typos

    info_ln "Taplo ...\n"
    cmd_taplo_check "$@"

    info_ln "Prettier ...\n"
    cmd_prettier_check "$@"

    info_ln "Typos ...\n"
    cmd_typos_check "$@"

    success_ln "CI Lint Succeeded.\n"

}
cmd_ci_clippy () {

    (( ENSURE_TOOLS )) && cmd_ensure clippy

    info_ln "Clippy ...\n"
    cmd_clippy "$@"

    success_ln "CI Clippy Succeeded.\n"

}

cmd_ci_audit () {

    (( ENSURE_TOOLS )) && cmd_ensure audit deny

    info_ln "Audit ...\n"
    cmd_audit_check "$@"

    success_ln "CI Audit Succeeded.\n"

}
cmd_ci_vet () {

    (( ENSURE_TOOLS )) && cmd_ensure vet

    info_ln "Vet ...\n"
    cmd_vet_check "$@"

    success_ln "CI Vet Succeeded.\n"

}
cmd_ci_hack () {

    (( ENSURE_TOOLS )) && cmd_ensure hack

    info_ln "Hack ...\n"
    cmd_hack "$@"

    success_ln "CI Hack Succeeded.\n"

}
cmd_ci_udeps () {

    (( ENSURE_TOOLS )) && cmd_ensure udeps

    info_ln "Udeps ...\n"
    cmd_udeps "$@"

    success_ln "CI Udeps Succeeded.\n"

}
cmd_ci_bloat () {

    (( ENSURE_TOOLS )) && cmd_ensure bloat

    info_ln "Bloat ...\n"
    cmd_bloat "$@"

    success_ln "CI Bloat Succeeded.\n"

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

    (( ENSURE_TOOLS )) && cmd_ensure fuzz

    info_ln "Fuzz ...\n"
    cmd_fuzz "$@"

    success_ln "CI Fuzz Succeeded.\n"

}
cmd_ci_miri () {

    (( ENSURE_TOOLS )) && cmd_ensure miri

    info_ln "Miri ...\n"
    cmd_miri "$@"

    success_ln "CI Miri Succeeded.\n"

}

cmd_ci_semver () {

    (( ENSURE_TOOLS )) && cmd_ensure semver

    info_ln "Semver ...\n"
    cmd_semver "$@"

    success_ln "CI Semver Succeeded.\n"

}
cmd_ci_coverage () {

    (( ENSURE_TOOLS )) && cmd_ensure cov

    info_ln "Coverage ...\n"
    cmd_coverage --upload "$@"

    success_ln "CI Coverage Succeeded.\n"

}
cmd_ci_publish () {

    info_ln "Publish ...\n"
    cmd_publish "$@"

    success_ln "CI Publish Succeeded.\n"

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
    cmd_ci_vet
    cmd_ci_hack
    cmd_ci_udeps

    cmd_ci_sanitizer
    cmd_ci_miri
    cmd_ci_semver

    cmd_ci_coverage --no-upload
    cmd_ci_publish --dry-run

    success_ln "CI Pipeline Succeeded.\n"

}
