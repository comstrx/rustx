#!/usr/bin/env bash

resolve_config () {

    local tool="${1-}" config="${2-}"
    shift 2 || true

    if [[ -n "${config}" ]]; then

        [[ -f "${config}" ]] || die "${tool}: config file not found: ${config}" 2

        printf '%s\n' "${config}"
        return 0

    fi

    config_file "$@"

}
vet_run () {

    local sub="${1-}"
    shift || true

    [[ -n "${sub}" ]] || die "vet_run: missing sub-command" 2

    cmd_vet_init
    run_cargo vet "${sub}" "$@"

}

cmd_safety_help () {

    info_ln "Safety :\n"

    printf '    %s\n' \
        "clippy                     * Clippy for publishable crates only (workspace gate)" \
        "clippy-strict              * Clippy for full workspace (including non-publishable crates)" \
        "" \
        "audit-check                * Security advisories gate (cargo deny advisories/bans/licenses/sources)" \
        "audit-fix                  * Auto-fix advisories by upgrading dependencies (cargo audit fix)" \
        "" \
        "sbom                       * Generate a checksummed CycloneDX SBOM with Syft" \
        "trivy                      * Scan vulnerabilities, secrets, misconfigurations, and licenses" \
        "leaks                      * Detect secrets with Gitleaks (git history in CI, files locally)" \
        "" \
        "udeps                      * Detect unused dependencies (cargo udeps)" \
        "hack                       * Feature-matrix checks (cargo hack)" \
        "semver                     * Semver compatibility checks (cargo semver-checks)" \
        "" \
        "fuzz                       * Fuzz targets (cargo fuzz) with sane defaults" \
        "sanitizer                  * Sanitizers pipeline (asan/tsan/msan/lsan) for UB detection; --no-build-std skips instrumenting std" \
        "miri                       * Miri interpreter checks (UB / unsafe issues)" \
        "" \
        "vet-init                   * Initialize supply-chain auditing (cargo vet init)" \
        "vet-fmt                    * Format supply-chain files (cargo vet fmt)" \
        "vet-check                  * Verify audits and policies (cargo vet check)" \
        "vet-suggest                * Suggest policy imports / criteria (cargo vet suggest)" \
        "" \
        "vet-diff                   * Diff between dependency versions (cargo vet diff)" \
        "vet-certify                * Certify a crate version into audits (cargo vet certify)" \
        "vet-trust                  * Trust a crate/publisher (cargo vet trust)" \
        "vet-deny                   * Record a violation (cargo vet record-violation)" \
        "vet-prune                  * Prune unused audits (cargo vet prune)" \
        "vet-renew                  * Renew audit freshness (cargo vet renew)" \
        "vet-clean                  * Delete supply-chain directory (cleanup)" \
        "vet-import                 * Import audits from <name> (cargo vet import)" \
        "" \
        "vet-import-best            * Import best presets (mozilla/google/isrg/bytecode-alliance)" \
        "vet-trust-best             * Apply curated trust set (safe-to-deploy baseline)" \
        ''

}

cmd_clippy () {

    run_workspace_publishable clippy features-on targets-on "$@"

}
cmd_clippy_strict () {

    run_workspace clippy features-on targets-on "$@"

}

cmd_audit_check () {

    run_cargo deny check advisories bans licenses sources "$@"

}
cmd_audit_fix () {

    local adv="${CARGO_HOME:-${HOME}/.cargo}/advisory-db"

    if [[ -d "${adv}" && ! -d "${adv}/.git" ]]; then
        run mv -f -- "${adv}" "${adv}.broken.$(date +%s)" || die "audit-fix: failed to move the non-git advisory-db aside: ${adv}" 2
    fi

    run_cargo audit fix "$@"

}

cmd_sbom () {

    ensure syft
    source <(parse "$@" -- src format out config name version supplier)

    src="${src:-dir:.}"
    format="${format:-cyclonedx-json}"
    out="${out:-out/sbom.json}"
    name="${name:-${ROOT_DIR##*/}}"

    if [[ -z "${version}" ]] && declare -F cmd_version >/dev/null 2>&1; then
        version="$(cmd_version 2>/dev/null || true)"
    fi

    local -a args=( scan )

    config="$(resolve_config syft "${config}" syft yml yaml)"

    [[ -f "${config}" ]] && args+=( --config "${config}" )
    [[ -n "${name}" ]] && args+=( --source-name "${name}" )
    [[ -n "${version}" ]] && args+=( --source-version "${version}" )
    [[ -n "${supplier}" ]] && args+=( --source-supplier "${supplier}" )

    if [[ "${out}" == "-" || "${out}" == "/dev/stdout" ]]; then
        args+=( --output "${format}" )
    else
        [[ "${out}" == */* ]] && ensure_dir "${out%/*}"
        args+=( --output "${format}=${out}" )
    fi

    run syft "${args[@]}" "${kwargs[@]}" -- "${src}"

}
cmd_trivy () {

    ensure trivy
    source <(parse "$@" -- mode format target out scanners severity config no_progress:bool=true ignore_unfixed:bool=false fail:bool=true)

    mode="${mode:-fs}"
    format="${format:-table}"
    target="${target:-.}"
    scanners="${scanners:-vuln,secret,misconfig,license}"

    case "${mode}" in
        fs|rootfs|image|repository|repo) ;;
        *) die "trivy: unsupported scan mode: ${mode}" 2 ;;
    esac

    local exit_code=0
    (( fail )) && exit_code=1

    local -a args=( "${mode}" --format "${format}" --exit-code "${exit_code}" )

    config="$(resolve_config trivy "${config}" trivy yml yaml)"

    [[ -f "${config}" ]] && args+=( --config "${config}" )
    [[ -n "${severity}" ]] && args+=( --severity "${severity}" )
    [[ -n "${scanners}" ]] && args+=( --scanners "${scanners}" )

    (( no_progress )) && args+=( --no-progress )
    (( ignore_unfixed )) && [[ "${scanners}" == *vuln* ]] && args+=( --ignore-unfixed )

    if [[ -n "${out}" && "${out}" != "-" && "${out}" != "/dev/stdout" ]]; then
        [[ "${out}" == */* ]] && ensure_dir "${out%/*}"
        args+=( --output "${out}" )
    fi

    run trivy "${args[@]}" "${kwargs[@]}" -- "${target}"

}
cmd_leaks () {

    ensure gitleaks
    source <(parse "$@" -- mode format target out config baseline redact=100 fail:bool=true)

    [[ -n "${mode}" ]] || { is_ci && mode="git" || mode="dir"; }
    format="${format:-json}"
    target="${target:-.}"
    out="${out:--}"

    case "${mode}" in
        git|dir) ;;
        *) die "gitleaks: unsupported scan mode: ${mode}" 2 ;;
    esac

    local exit_code=0
    (( fail )) && exit_code=1

    local -a args=( "${mode}" --no-banner --no-color --report-format "${format}" --exit-code "${exit_code}" )

    config="$(resolve_config gitleaks "${config}" gitleaks toml)"

    [[ -f "${config}" ]] && args+=( --config "${config}" )
    [[ -n "${baseline}" ]] && args+=( --baseline-path "${baseline}" )
    [[ -n "${redact}" ]] && args+=( --redact="${redact}" )

    [[ "${out}" == "/dev/stdout" ]] && out="-"
    if [[ -n "${out}" ]]; then
        [[ "${out}" != "-" && "${out}" == */* ]] && ensure_dir "${out%/*}"
        args+=( --report-path "${out}" )
    fi

    run gitleaks "${args[@]}" "${kwargs[@]}" -- "${target}"

}

cmd_udeps () {

    run_cargo udeps --nightly --all-targets "$@"

}
cmd_hack () {

    source <(parse "$@" -- depth:int=2 each_feature:bool)

    if (( each_feature )); then
        run_cargo hack check --keep-going --each-feature "${kwargs[@]}"
        return $?
    fi

    run_cargo hack check --keep-going --feature-powerset --depth "${depth}" "${kwargs[@]}"

}
cmd_semver () {

    source <(parse "$@" -- baseline remote=origin)

    if [[ -z "${baseline}" ]]; then

        if is_ci_pull; then

            local base="${GITHUB_BASE_REF:-}"
            [[ -n "${base}" ]] || die "semver: missing GITHUB_BASE_REF. Provide --baseline <rev>." 2

            run git fetch --no-tags "${remote}" "${base}:refs/remotes/${remote}/${base}" >/dev/null 2>&1 || die "semver: failed to fetch." 2
            baseline="${remote}/${base}"

        elif is_ci_push; then

            run git fetch --tags --force --prune "${remote}" >/dev/null 2>&1 || true

            local cur="${GITHUB_REF_NAME:-}"

            baseline="$(
                git tag --list 'v*' --sort=-v:refname |
                grep -E '^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$' |
                grep -F -x -v -- "${cur}" |
                head -n 1 || true
            )"

            if [[ -z "${baseline}" ]]; then
                log "semver: first stable release -> Skipping."
                return 0
            fi

        else

            local def=""
            def="$(git symbolic-ref -q "refs/remotes/${remote}/HEAD" 2>/dev/null || true)"
            def="${def#refs/remotes/${remote}/}"
            [[ -n "${def}" ]] || def="main"

            run git fetch --no-tags "${remote}" "${def}:refs/remotes/${remote}/${def}" >/dev/null 2>&1 || true

            if git show-ref --verify --quiet "refs/remotes/${remote}/${def}"; then
                baseline="${remote}/${def}"
            else
                log "semver: no baseline branch found (${remote}/${def}) -> Skipping."
                return 0
            fi

        fi
    fi

    [[ -n "${baseline}" ]] || { log "semver: no baseline. Skipping."; return 0; }
    git rev-parse --verify "${baseline}^{commit}" >/dev/null 2>&1 || die "semver: baseline '${baseline}' is not a valid." 2

    local -a extra=()
    local help_out=""

    help_out="$(run_cargo semver-checks -h 2>/dev/null || true)"
    [[ "${help_out}" == *"--baseline-rev"* ]] && extra+=( --baseline-rev "${baseline}" )

    run_cargo semver-checks "${extra[@]}" "${kwargs[@]}"

}

cmd_fuzz () {

    source <(parse "$@" -- timeout:int=10 len:int=4096 have_max_total_time:bool have_max_len:bool in_post:bool)

    local -a pre=() post=()

    while [[ $# -gt 0 ]]; do

        if [[ "$1" == "--" ]]; then
            in_post=1
            shift || true
            continue
        fi
        if (( in_post )); then
            case "$1" in
                -max_total_time|-max_total_time=*) have_max_total_time=1 ;;
                -max_len|-max_len=*) have_max_len=1 ;;
            esac
            post+=( "$1" )
            shift || true
            continue
        fi
        case "$1" in
            --timeout) shift || true; [[ $# -gt 0 ]] || die "Missing value for --timeout" 2; timeout="$1"; shift || true ;;
            --timeout=*) timeout="${1#*=}"; shift || true ;;
            --len) shift || true; [[ $# -gt 0 ]] || die "Missing value for --len" 2; len="$1"; shift || true ;;
            --len=*) len="${1#*=}"; shift || true ;;
            -max_total_time|-max_total_time=*) have_max_total_time=1; post+=( "$1" ); shift || true ;;
            -max_len|-max_len=*) have_max_len=1; post+=( "$1" ); shift || true ;;
            *) pre+=( "$1" ); shift || true ;;
        esac

    done

    if [[ -z "${CARGO_BUILD_TARGET:-}" ]] || [[ "${CARGO_BUILD_TARGET:-}" == *-musl ]]; then

        local fuzz_target=""
        fuzz_target="$(host_triple "$(nightly_version)")"

        [[ "${fuzz_target}" == *-musl ]] && fuzz_target="${fuzz_target%-musl}-gnu"

        pre+=( "--target" "${fuzz_target}" )

    fi
    if [[ "${#pre[@]}" -eq 0 ]] || [[ "${pre[0]-}" == -* ]]; then

        (( have_max_total_time )) || [[ "${timeout}" == "0" ]] || post+=( "-max_total_time=${timeout}" )
        (( have_max_len )) || [[ "${len}" == "0" ]] || post+=( "-max_len=${len}" )

        local -a targets=()
        local t="" line=""

        while IFS= read -r line; do
            [[ -n "${line}" ]] || continue
            targets+=( "${line}" )
        done < <(run_cargo fuzz --nightly list 2>/dev/null || true)

        [[ "${#targets[@]}" -gt 0 ]] || die "No fuzz targets found. Run: cargo fuzz init && cargo fuzz add <name>" 2

        for t in "${targets[@]}"; do

            if [[ "${#post[@]}" -gt 0 ]]; then run_cargo fuzz --nightly run "${t}" "${pre[@]}" -- "${post[@]}" || die "Fuzzing failed: ${t}" 2
            else run_cargo fuzz --nightly run "${t}" "${pre[@]}" || die "Fuzzing failed: ${t}" 2
            fi

        done

        return 0

    fi
    if [[ "${#pre[@]}" -gt 0 ]]; then

        case "${pre[0]}" in
            run|list|init|add|clean|cmin|tmin|coverage|fmt) ;;
            *) pre=( "run" "${pre[@]}" ) ;;
        esac

    fi
    if [[ "${pre[0]}" == "run" ]]; then

        (( have_max_total_time )) || [[ "${timeout}" == "0" ]] || post+=( "-max_total_time=${timeout}" )
        (( have_max_len )) || [[ "${len}" == "0" ]] || post+=( "-max_len=${len}" )

    fi
    if [[ "${#post[@]}" -gt 0 ]]; then

        run_cargo fuzz --nightly "${pre[@]}" -- "${post[@]}"
        return $?

    fi

    run_cargo fuzz --nightly "${pre[@]}"

}
sanitizer_build_std_ready () {

    local tc="${1-}" sysroot="" lib=""

    sysroot="$(rust_sysroot "${tc}")" || return 1
    lib="${sysroot}/lib/rustlib/src/rust/library"

    [[ -d "${lib}" ]] || return 1
    [[ -f "${lib}/Cargo.lock" ]] || return 1

    return 0

}
cmd_sanitizer () {

    source <(parse "$@" -- :sanitizer=asan command=test :target=auto clean:bool=0 track_origins:bool=1 build_std:bool=1)

    local target="${target}" san="${sanitizer}" zsan="" opt="" tc=""
    local -a extra=()

    tc="$(nightly_version)"

    case "${san}" in
        asan|address)      san="asan"  ; zsan="address" ;;
        tsan|thread)       san="tsan"  ; zsan="thread" ;;
        lsan|leak)         san="lsan"  ; zsan="leak" ;;
        msan|memory)
            san="msan"
            zsan="memory"
            (( track_origins )) && extra+=( "-Zsanitizer-memory-track-origins" )
        ;;
        *) die "sanitizer: unknown sanitizer '${sanitizer}' (use: asan|tsan|msan|lsan)" 2 ;;
    esac

    if [[ -z "${target}" || "${target}" == "auto" ]]; then
        target="$(host_triple "${tc}")"
    fi

    local target_dir="target/sanitizers/${san}"
    local rf="${RUSTFLAGS:-}"
    local rdf="${RUSTDOCFLAGS:-}"

    [[ -n "${rf}" ]] && rf+=" "
    [[ -n "${rdf}" ]] && rdf+=" "

    rf+="-Zsanitizer=${zsan} -Cforce-frame-pointers=yes -Cdebuginfo=1"
    rdf+="-Zsanitizer=${zsan} -Cforce-frame-pointers=yes -Cdebuginfo=1"

    for opt in "${extra[@]}"; do
        rf+=" ${opt}"
        rdf+=" ${opt}"
    done

    (( clean )) && { CARGO_TARGET_DIR="${target_dir}" run_cargo clean --nightly --target "${target}" >/dev/null 2>&1 || true; }
    log "=> sanitizer: ${san} (-Zsanitizer=${zsan}) target=${target} command=${command} \n"

    local -a std_args=()

    if (( build_std )); then

        sanitizer_build_std_ready "${tc}" || die "sanitizer: toolchain ${tc} has no rust-src component; run 'rustup component add rust-src --toolchain ${tc}', or re-run with --no-build-std." 2

        std_args+=( -Zbuild-std=std )

    else
        warn "sanitizer: --no-build-std - std is NOT instrumented, so defects inside std will be missed."
    fi

    local log="" err_trap="" code=0

    fs_tmp_file log "sanitizer.XXXXXXXX"
    err_trap="$(trap -p ERR 2>/dev/null || true)"

    trap - ERR
    set +e

    CARGO_TARGET_DIR="${target_dir}" \
        CARGO_INCREMENTAL=0 \
        RUSTFLAGS="${rf}" \
        RUSTDOCFLAGS="${rdf}" \
        run_cargo "${command}" --nightly "${std_args[@]}" --target "${target}" "${kwargs[@]}" 2> >(tee -- "${log}" >&2)

    code=$?

    # The tee procsub flushes ${log} asynchronously; wait for it before grepping.
    wait "$!" 2>/dev/null || true

    set -e
    [[ -n "${err_trap}" ]] && eval "${err_trap}"

    if (( code != 0 )) && (( build_std )) && grep -q -- 'in path source' "${log}"; then

        fs_tmp_release "${log}"
        die "sanitizer: toolchain ${tc} cannot resolve std's own dependencies for -Zbuild-std (upstream cargo/rust-src defect, reproducible on an empty crate). Update the nightly toolchain, or re-run with --no-build-std to instrument workspace code only." 2

    fi

    fs_tmp_release "${log}"

    return "${code}"

}
cmd_miri () {

    source <(parse "$@" -- command=test :target=auto clean:bool setup:bool=1)

    local target="${target}" tc=""
    local target_dir="target/miri"

    tc="$(nightly_version)"

    if [[ -z "${target}" || "${target}" == "auto" ]]; then
        target="$(host_triple "${tc}")"
    fi

    (( clean )) && { CARGO_TARGET_DIR="${target_dir}" run_cargo clean --nightly --target "${target}" >/dev/null 2>&1 || true; }
    (( setup )) && { CARGO_TARGET_DIR="${target_dir}" run_cargo miri --nightly setup >/dev/null 2>&1 || true; }

    CARGO_TARGET_DIR="${target_dir}" CARGO_INCREMENTAL=0 run_cargo miri --nightly "${command}" --target "${target}" "${kwargs[@]}"

}

cmd_vet_init () {

    [[ -f "${ROOT_DIR}/Cargo.lock" ]] || run_cargo generate-lockfile
    [[ -f "${ROOT_DIR}/supply-chain/config.toml" && -f "${ROOT_DIR}/supply-chain/audits.toml" ]] || run_cargo vet init

}
cmd_vet_fmt () {

    vet_run fmt "$@"

}
cmd_vet_check () {

    vet_run check "$@"

}
cmd_vet_suggest () {

    vet_run suggest "$@"

}

cmd_vet_diff () {

    vet_run diff "$@"

}
cmd_vet_certify () {

    source <(parse "$@" -- :name :version :criteria="safe-to-run")

    vet_run certify "${name}" "${version}" --criteria "${criteria}" "${kwargs[@]}"

}
cmd_vet_trust () {

    vet_run trust "$@"

}
cmd_vet_deny () {

    vet_run record-violation "$@"

}
cmd_vet_prune () {

    vet_run prune "$@"

}
cmd_vet_renew () {

    vet_run renew "$@"

}
cmd_vet_clean () {

    confirm "Are you sure about deleting the supply chain?" || return 0

    remove_dir "${ROOT_DIR}/supply-chain"

}
cmd_vet_import () {

    source <(parse "$@" -- :name)

    vet_run import "${name}" "${kwargs[@]}"

}

cmd_vet_import_best () {

    cmd_vet_import mozilla
    cmd_vet_import google
    cmd_vet_import isrg
    cmd_vet_import bytecode-alliance

}
cmd_vet_trust_best () {

    cmd_vet_trust --all dtolnay                           --criteria safe-to-deploy
    cmd_vet_trust r-efi dvdhrm                            --criteria safe-to-deploy
    cmd_vet_trust libfuzzer-sys fitzgen                   --criteria safe-to-deploy
    cmd_vet_trust getrandom josephlr                      --criteria safe-to-deploy
    cmd_vet_trust find-msvc-tools cuviper                 --criteria safe-to-deploy
    cmd_vet_trust libc rust-lang-owner                    --criteria safe-to-deploy
    cmd_vet_trust jobserver rust-lang-owner               --criteria safe-to-deploy
    cmd_vet_trust cc github:rust-lang/cc-rs               --criteria safe-to-deploy
    cmd_vet_trust find-msvc-tools github:rust-lang/cc-rs  --criteria safe-to-deploy

}
