#!/usr/bin/env bash

cmd_safety_help () {

    info_ln "Safety :\n"

    printf '    %s\n' \
        "clippy                     * Clippy for publishable crates only (workspace gate)" \
        "clippy-strict              * Clippy for full workspace (including non-publishable crates)" \
        "" \
        "audit-check                * Security advisories gate (cargo deny advisories/bans/licenses/sources)" \
        "audit-fix                  * Auto-fix advisories by upgrading dependencies (cargo audit fix)" \
        "" \
        "sbom, syft                 * Generate a checksummed CycloneDX SBOM with Syft" \
        "trivy                      * Scan vulnerabilities, secrets, misconfigurations, and licenses" \
        "leaks                      * Detect secrets with Gitleaks (git history in CI, files locally)" \
        "" \
        "udeps                      * Detect unused dependencies (cargo udeps)" \
        "hack                       * Feature-matrix checks (cargo hack)" \
        "semver                     * Semver compatibility checks (cargo semver-checks)" \
        "" \
        "fuzz                       * Fuzz targets (cargo fuzz) with sane defaults" \
        "sanitizer                  * Sanitizers pipeline (asan/tsan/msan/lsan) for UB detection" \
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

    local adv="${HOME}/.cargo/advisory-db"
    [[ -d "${adv}" ]] && [[ ! -d "${adv}/.git" ]] && mv "${adv}" "${adv}.broken.$(date +%s)" || true

    run_cargo audit fix "$@"

}

cmd_sbom () {

    ensure syft
    source <(parse "$@" -- src format out config name version supplier)

    src="${src:-dir:.}"
    format="${format:-cyclonedx-json}"
    out="${out:-${OUT_DIR:-out}/sbom.json}"
    name="${name:-${ROOT_DIR##*/}}"

    if [[ -z "${version}" ]] && declare -F cmd_version >/dev/null 2>&1; then
        version="$(cmd_version 2>/dev/null || true)"
    fi

    local -a args=( scan )

    if [[ -n "${config}" ]]; then
        [[ -f "${config}" ]] || die "syft: config file not found: ${config}" 2
    else
        config="$(config_file syft yaml yml)"
    fi

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
cmd_syft () {

    cmd_sbom "$@"

}
cmd_trivy () {

    ensure trivy
    source <(parse "$@" -- mode format target out scanners severity config no_progress:bool=true ignore_unfixed:bool=true fail:bool=true)

    mode="${mode:-fs}"
    format="${format:-table}"
    target="${target:-.}"
    scanners="${scanners:-vuln,secret,misconfig,license}"
    severity="${severity:-CRITICAL,HIGH}"

    case "${mode}" in
        fs|rootfs|image|repository|repo) ;;
        *) die "trivy: unsupported scan mode: ${mode}" 2 ;;
    esac

    local exit_code=0
    (( fail )) && exit_code=1

    local -a args=( "${mode}" --format "${format}" --exit-code "${exit_code}" )

    if [[ -n "${config}" ]]; then
        [[ -f "${config}" ]] || die "trivy: config file not found: ${config}" 2
    else
        config="$(config_file trivy yaml yml)"
    fi

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

    if [[ -n "${config}" ]]; then
        [[ -f "${config}" ]] || die "gitleaks: config file not found: ${config}" 2
    else
        config="$(config_file gitleaks toml)"
    fi

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
        return 0
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

            local def="$(git symbolic-ref -q "refs/remotes/${remote}/HEAD" 2>/dev/null || true)"
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
    run_cargo semver-checks -h 2>/dev/null | grep -q -- '--baseline-rev' && extra+=(--baseline-rev "${baseline}")

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

        pre+=( "--target" "x86_64-unknown-linux-gnu" )

    fi
    if [[ "${#pre[@]}" -eq 0 ]] || [[ "${pre[0]-}" == -* ]]; then

        (( have_max_total_time )) || [[ "${timeout}" == "0" ]] || post+=( "-max_total_time=${timeout}" )
        (( have_max_len )) || [[ "${len}" == "0" ]] || post+=( "-max_len=${len}" )

        local -a targets=()
        local t=""

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
cmd_sanitizer () {

    source <(parse "$@" -- :sanitizer=asan command=test :target=auto clean:bool=0 track_origins:bool=1)

    local target="${target}" san="${sanitizer}" zsan="" opt="" tc="$(nightly_version)"
    local -a extra=()

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

        local vv="$(rustc +"${tc}" -vV 2>/dev/null)" || die "sanitizer: failed to read rustc -vV for ${tc}" 2
        target="$(awk '/^host: / { print $2; exit }' <<< "${vv}")"
        [[ -n "${target}" ]] || die "sanitizer: failed to detect host target." 2

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

    CARGO_TARGET_DIR="${target_dir}" \
        CARGO_INCREMENTAL=0 \
        RUSTFLAGS="${rf}" \
        RUSTDOCFLAGS="${rdf}" \
        run_cargo "${command}" --nightly -Zbuild-std=std --target "${target}" "${kwargs[@]}"

}
cmd_miri () {

    source <(parse "$@" -- command=test :target=auto clean:bool setup:bool=1)

    local target="${target}" tc="$(nightly_version)"
    local target_dir="target/miri"

    if [[ -z "${target}" || "${target}" == "auto" ]]; then

        local vv="$(rustc +"${tc}" -vV 2>/dev/null)" || die "miri: failed to read rustc -vV for ${tc}" 2
        target="$(awk '/^host: / { print $2; exit }' <<< "${vv}")"
        [[ -n "${target}" ]] || die "miri: failed to detect host target." 2

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

    cmd_vet_init
    run_cargo vet fmt "$@"

}
cmd_vet_check () {

    cmd_vet_init
    run_cargo vet check "$@"

}
cmd_vet_suggest () {

    cmd_vet_init
    run_cargo vet suggest "$@"

}

cmd_vet_diff () {

    cmd_vet_init
    run_cargo vet diff "$@"

}
cmd_vet_certify () {

    source <(parse "$@" -- :name :version :criteria="safe-to-run")

    cmd_vet_init
    run_cargo vet certify "${name}" "${version}" --criteria "${criteria}" "${kwargs[@]}"

}
cmd_vet_trust () {

    cmd_vet_init
    run_cargo vet trust "$@"

}
cmd_vet_deny () {

    cmd_vet_init
    run_cargo vet record-violation "$@"

}
cmd_vet_prune () {

    cmd_vet_init
    run_cargo vet prune "$@"

}
cmd_vet_renew () {

    cmd_vet_init
    run_cargo vet renew "$@"

}
cmd_vet_clean () {

    confirm "Are you sure about deleting the supply chain?" && rm -rf supply-chain

}
cmd_vet_import () {

    source <(parse "$@" -- :name)

    cmd_vet_init
    run_cargo vet import "${name}" "${kwargs[@]}"

}

cmd_vet_import_best () {

    cmd_vet_import mozilla
    cmd_vet_import google
    cmd_vet_import isrg
    cmd_vet_import bytecode-alliance

}
cmd_vet_trust_best () {

    cmd_vet_trust dtolnay                                 --criteria safe-to-deploy
    cmd_vet_trust r-efi dvdhrm                            --criteria safe-to-deploy
    cmd_vet_trust libfuzzer-sys fitzgen                   --criteria safe-to-deploy
    cmd_vet_trust getrandom josephlr                      --criteria safe-to-deploy
    cmd_vet_trust find-msvc-tools cuviper                 --criteria safe-to-deploy
    cmd_vet_trust libc rust-lang-owner                    --criteria safe-to-deploy
    cmd_vet_trust jobserver rust-lang-owner               --criteria safe-to-deploy
    cmd_vet_trust cc github:rust-lang/cc-rs               --criteria safe-to-deploy
    cmd_vet_trust find-msvc-tools github:rust-lang/cc-rs  --criteria safe-to-deploy

}
