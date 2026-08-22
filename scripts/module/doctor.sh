#!/usr/bin/env bash

DOCTOR_OK=0
DOCTOR_WARN=0
DOCTOR_FAIL=0

doctor_pick_ver_line () {

    local s="${1-}"
    local line=""

    while IFS= read -r line; do

        line="${line//$'\r'/}"
        line="${line#"${line%%[!$' \t']*}"}"
        line="${line%"${line##*[!$' \t']}"}"

        [[ -n "${line}" ]] || continue
        [[ "${line}" =~ [0-9]+\.[0-9]+ ]] && { printf '%s' "${line}"; return 0; }

    done <<< "${s}"

    while IFS= read -r line; do

        line="${line//$'\r'/}"
        line="${line#"${line%%[!$' \t']*}"}"
        line="${line%"${line##*[!$' \t']}"}"

        [[ -n "${line}" ]] || continue
        printf '%s' "${line}"
        return 0

    done <<< "${s}"

    printf '%s' ""
    return 0

}
doctor_ver () {

    ensure head tr

    local out=""
    out="$( { "$@" --version 2>&1 || "$@" -V 2>&1 || "$@" version 2>&1 || true; } | head -n 6 | tr -d '\r' )"

    out="$(doctor_pick_ver_line "${out}")"
    [[ -n "${out}" ]] && { printf '%s' "${out}"; return 0; }

    printf '%s' ""
    return 0

}
doctor_status () {

    local kind="${1:-ok}" name="${2:-}" msg="${3:-}"

    case "${kind}" in
        warn)
            printf '  ⚠️ %-18s %s\n' "${name}" "${msg}"
            DOCTOR_WARN=$(( DOCTOR_WARN + 1 ))
        ;;
        fail)
            printf '  ❌ %-18s %s\n' "${name}" "${msg}"
            DOCTOR_FAIL=$(( DOCTOR_FAIL + 1 ))
        ;;
        *)
            printf '  ✅ %-18s %s\n' "${name}" "${msg}"
            DOCTOR_OK=$(( DOCTOR_OK + 1 ))
        ;;
    esac

}
doctor_tool () {

    local missing_kind="${1:-warn}"
    local name="${2:-}"
    local cmd="${3:-}"

    shift 3 || true

    if ! has "${cmd}"; then
        doctor_status "${missing_kind}" "${name}" "missing"
        return 0
    fi

    local v=""
    v="$(doctor_ver "${cmd}" "$@")"

    if [[ -z "${v}" ]]; then
        doctor_status warn "${name}" "unknown"
        return 0
    fi
    if [[ "${v}" == *"error:"* || "${v}" == *"fatal error:"* ]]; then
        doctor_status warn "${name}" "${v}"
        return 0
    fi

    doctor_status ok "${name}" "${v}"
    return 0

}
doctor_cargo_sub () {

    local label="${1:-}"
    local sub="${2:-}"
    local bin="${3:-}"

    if ! has cargo; then
        doctor_status fail "cargo" "missing"
        return 0
    fi
    if ! has "${bin}"; then
        doctor_status warn "${label}" "missing"
        return 0
    fi

    local v=""
    v="$(doctor_ver cargo "${sub}")"

    [[ -n "${v}" ]] || { doctor_status warn "${label}" "unknown"; return 0; }

    if [[ "${v}" == *"error:"* || "${v}" == *"fatal error:"* ]]; then
        doctor_status warn "${label}" "${v}"
        return 0
    fi

    doctor_status ok "${label}" "${v}"
    return 0

}
doctor_has_component () {

    ensure awk grep

    local tc="${1:-}"
    local comp_re="${2:-}"

    rustup component list --toolchain "${tc}" --installed 2>/dev/null | awk '{print $1}' | grep -qE "${comp_re}"

}
doctor_toolchain_installed () {

    ensure awk

    local tc="${1:-}"
    [[ -n "${tc}" ]] || return 1

    rustup toolchain list 2>/dev/null \
        | awk '{print $1}' \
        | awk -v tc="${tc}" '
            $0 == tc { found=1 }
            index($0, tc "-") == 1 { found=1 }
            END { exit(found ? 0 : 1) }
        '

}
doctor_toolchain () {

    local name="${1:-}"
    local tc="${2:-}"

    [[ -n "${tc}" ]] || { doctor_status warn "${name}" "unknown"; return 0; }

    if doctor_toolchain_installed "${tc}"; then
        doctor_status ok "${name}" "${tc}"
        return 0
    fi

    doctor_status warn "${name}" "missing (${tc})"
    return 0

}
doctor_sys () {

    ensure awk grep

    local distro="unknown" wsl="No" ci="No" shell="${SHELL:-unknown}"
    local os="" kernel="" arch=""

    is_ci && ci="Yes"
    is_wsl && wsl="Yes"

    os="$(os_name)"
    kernel="$(uname -r 2>/dev/null || printf '%s' unknown)"
    arch="$(uname -m 2>/dev/null || printf '%s' unknown)"

    if [[ -r /etc/os-release ]]; then
        distro="$(. /etc/os-release 2>/dev/null; printf '%s' "${PRETTY_NAME:-unknown}")"
    elif [[ "${os}" == "mac" ]]; then
        distro="macOS $(sw_vers -productVersion 2>/dev/null || printf '%s' unknown)"
    elif [[ "${os}" == "windows" ]]; then
        distro="Windows ($(uname -o 2>/dev/null || uname -s 2>/dev/null || printf '%s' unknown))"
    fi

    local cpu="" cores="" mem="" disk=""

    if has lscpu; then
        cpu="$(lscpu 2>/dev/null | awk -F: '/Model name/ { sub(/^[ \t]+/,"",$2); print $2; exit }' || true)"
    fi
    if [[ -z "${cpu}" && -r /proc/cpuinfo ]]; then
        cpu="$(awk -F: '/model name/ { sub(/^[ \t]+/,"",$2); print $2; exit }' /proc/cpuinfo 2>/dev/null || true)"
    fi
    if [[ -z "${cpu}" && "${os}" == "mac" ]] && has sysctl; then
        cpu="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || true)"
    fi
    if [[ -z "${cpu}" && "${os}" == "windows" ]]; then
        cpu="${PROCESSOR_IDENTIFIER:-}"
    fi
    [[ -n "${cpu}" ]] || cpu="unknown"

    if has nproc; then
        cores="$(nproc 2>/dev/null || true)"
    fi
    if [[ -z "${cores}" ]] && has getconf; then
        cores="$(getconf _NPROCESSORS_ONLN 2>/dev/null || true)"
    fi
    if [[ -z "${cores}" && "${os}" == "mac" ]] && has sysctl; then
        cores="$(sysctl -n hw.ncpu 2>/dev/null || true)"
    fi
    if [[ -z "${cores}" && "${os}" == "windows" ]]; then
        cores="${NUMBER_OF_PROCESSORS:-}"
    fi
    [[ "${cores}" =~ ^[0-9]+$ ]] || cores="unknown"

    if has free; then
        mem="$(free -h 2>/dev/null | awk '/^Mem:/ { print $2 " total, " $7 " avail"; exit }' || true)"
    fi
    if [[ -z "${mem}" && -r /proc/meminfo ]]; then
        mem="$(awk '
            /^MemTotal:/ { total = $2 }
            /^MemAvailable:/ { avail = $2 }
            /^MemFree:/ { unused = $2 }
            END {
                if (!total) exit 1
                if (!avail) avail = unused
                printf "%.1fGi total, %.1fGi avail", total / 1048576, avail / 1048576
            }
        ' /proc/meminfo 2>/dev/null || true)"
    fi
    if [[ -z "${mem}" && "${os}" == "mac" ]] && has sysctl; then

        local mem_bytes=""
        mem_bytes="$(sysctl -n hw.memsize 2>/dev/null || true)"

        [[ "${mem_bytes}" =~ ^[0-9]+$ ]] && mem="$(( mem_bytes / 1024 / 1024 ))Mi total"

    fi
    [[ -n "${mem}" ]] || mem="unknown"

    disk="$(LC_ALL=C df -hP . 2>/dev/null | awk 'NR==2 { print $4 " free of " $2 " (" $5 " used)"; exit }' || true)"
    [[ -n "${disk}" ]] || disk="unknown"

    info_ln '==> OS \n'

    doctor_status ok "OS" "${os}"
    doctor_status ok "Distro" "${distro}"
    doctor_status ok "Kernel" "${kernel}"
    doctor_status ok "Disk" "${disk}"
    doctor_status ok "CPU" "${cpu}"
    doctor_status ok "Memory" "${mem}"
    doctor_status ok "Shell" "${shell}"
    doctor_status ok "Cores" "${cores}"
    doctor_status ok "Arch" "${arch}"
    doctor_status ok "WSL" "${wsl}"
    doctor_status ok "CI" "${ci}"

}
doctor_github () {

    info_ln '==> Github \n'

    local root=""
    root="$(pwd -P 2>/dev/null || pwd 2>/dev/null || printf '%s' '.')"

    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then

        local has_commit=0 branch="unknown" head="unknown" dirty="n/a" origin="missing"

        root="$(git rev-parse --show-toplevel 2>/dev/null || printf '%s' "${root}")"

        origin="$(git remote get-url origin 2>/dev/null || true)"
        [[ -n "${origin}" ]] || origin="$(git remote get-url upstream 2>/dev/null || true)"

        if git rev-parse --verify HEAD >/dev/null 2>&1; then has_commit=1; fi

        if (( has_commit )); then
            branch="$(git symbolic-ref -q --short HEAD 2>/dev/null || true)"
            [[ -n "${branch}" ]] || branch="detached"
        else
            branch="unborn"
        fi

        if (( has_commit )); then head="$(git rev-parse --short HEAD 2>/dev/null || printf '%s' unknown)"
        else head="none"
        fi

        if (( has_commit )); then
            dirty="dirty"
            if git diff --quiet 2>/dev/null && git diff --cached --quiet 2>/dev/null; then dirty="clean"; fi
        else
            dirty="unborn"
        fi

        doctor_status ok "Root" "${root}"
        [[ -n "${origin}" ]] && doctor_status ok "Origin" "${origin}" || doctor_status warn "Origin" "missing"
        [[ "${branch}" == "unknown" || "${branch}" == "unborn" || "${branch}" == "detached" ]] && doctor_status warn "Branch" "${branch}" || doctor_status ok "Branch" "${branch}"
        [[ "${head}" == "unknown" || "${head}" == "none" ]] && doctor_status warn "Commit" "${head}" || doctor_status ok "Commit" "${head}"
        [[ "${dirty}" == "clean" ]] && doctor_status ok "Status" "${dirty}" || doctor_status warn "Status" "${dirty}"

    else
        doctor_status ok   "Root"   "${root}"
        doctor_status warn "Origin" "missing"
        doctor_status warn "Branch" "none"
        doctor_status warn "Commit" "none"
        doctor_status warn "Status" "n/a"
    fi

}
doctor_tools () {

    info_ln '==> Tools \n'

    doctor_tool warn "git" git
    doctor_tool warn "gh" gh
    doctor_tool warn "rustup" rustup
    doctor_tool warn "rustc" rustc
    doctor_tool warn "cargo" cargo
    doctor_tool warn "clang" clang
    doctor_tool warn "llvm" llvm-config
    doctor_tool warn "node" node
    doctor_tool warn "npx" npx
    doctor_tool warn "npm" npm

}
doctor_rust () {

    ensure awk wc tr
    info_ln '==> Rust \n'

    if ! has rustup; then
        doctor_status fail "rustup" "missing"
        return 0
    fi
    if ! has cargo; then
        doctor_status fail "cargo" "missing"
        return 0
    fi

    [[ -n "${RUSTFLAGS:-}" ]] && doctor_status ok "RUSTFLAGS" "${RUSTFLAGS}" || doctor_status ok "RUSTFLAGS" "--"
    [[ -n "${RUST_BACKTRACE:-}" ]] && doctor_status ok "RUST_BACKTRACE" "${RUST_BACKTRACE}" || doctor_status ok "RUST_BACKTRACE" "--"

    local active_tc="" stable_tc="" nightly_tc="" msrv_tc=""

    active_tc="$(rustup show active-toolchain 2>/dev/null | awk '{print $1}' || true)"
    [[ -n "${active_tc}" ]] && doctor_status ok "active" "${active_tc}" || doctor_status warn "active" "unknown"

    stable_tc="$(stable_version 2>/dev/null || true)"
    nightly_tc="$(nightly_version 2>/dev/null || true)"
    msrv_tc="$(msrv_version 2>/dev/null || true)"

    [[ -n "${stable_tc}" ]] || stable_tc="${RUST_STABLE:-stable}"
    [[ -n "${nightly_tc}" ]] || nightly_tc="${RUST_NIGHTLY:-nightly}"

    doctor_toolchain "stable" "${stable_tc}"
    doctor_toolchain "nightly" "${nightly_tc}"
    [[ -n "${msrv_tc}" ]] && doctor_toolchain "msrv" "${msrv_tc}" || doctor_status warn "msrv" "--"

    echo

    if doctor_has_component "${active_tc}" '^(llvm-tools|llvm-tools-preview)($|-)'; then

        local sysroot="" host="" bin=""

        sysroot="$(rust_sysroot "${active_tc}" || true)"
        host="$(host_triple "${active_tc}" 2>/dev/null || true)"
        bin="${sysroot}/lib/rustlib/${host}/bin/llvm-cov"

        [[ -x "${bin}" ]] || bin="${bin}.exe"

        if [[ -x "${bin}" ]]; then

            local v=""
            v="$( { "${bin}" --version 2>&1 || true; } | head -n 6 | tr -d '\r' )"
            v="$(doctor_pick_ver_line "${v}")"

            [[ -n "${v}" ]] && doctor_status ok "llvm-tools" "${v}" || doctor_status warn "llvm-tools" "unknown"

        else
            doctor_status warn "llvm-tools" "unknown"
        fi

    else
        doctor_status warn "llvm-tools" "missing"
    fi

    if doctor_toolchain_installed "${nightly_tc}" && doctor_has_component "${nightly_tc}" '^miri($|-)'; then doctor_tool warn "miri" cargo "+${nightly_tc}" miri
    else doctor_status warn "miri" "missing"
    fi

    if doctor_has_component "${active_tc}" '^rustfmt($|-)'; then doctor_tool warn "rustfmt" rustup run "${active_tc}" rustfmt
    else doctor_status warn "rustfmt" "missing"
    fi

    if doctor_has_component "${active_tc}" '^clippy($|-)'; then doctor_tool warn "clippy" rustup run "${active_tc}" clippy-driver
    else doctor_status warn "clippy" "missing"
    fi

    doctor_tool warn taplo taplo
    doctor_tool warn typos typos
    doctor_tool warn syft syft
    doctor_tool warn trivy trivy
    doctor_tool warn gitleaks gitleaks
    doctor_tool warn samply samply
    echo

    doctor_cargo_sub "binstall"         "binstall"       "cargo-binstall"
    doctor_cargo_sub "nextest"          "nextest"        "cargo-nextest"
    doctor_cargo_sub "llvm-cov"         "llvm-cov"       "cargo-llvm-cov"
    doctor_cargo_sub "flamegraph"       "flamegraph"     "cargo-flamegraph"
    echo
    doctor_cargo_sub "cargo-deny"       "deny"           "cargo-deny"
    doctor_cargo_sub "cargo-audit"      "audit"          "cargo-audit"
    doctor_cargo_sub "cargo-semver"     "semver-checks"  "cargo-semver-checks"
    doctor_cargo_sub "cargo-hack"       "hack"           "cargo-hack"
    doctor_cargo_sub "cargo-fuzz"       "fuzz"           "cargo-fuzz"
    doctor_cargo_sub "cargo-udeps"      "udeps"          "cargo-udeps"
    doctor_cargo_sub "cargo-bloat"      "bloat"          "cargo-bloat"
    doctor_cargo_sub "cargo-vet"        "vet"            "cargo-vet"
    doctor_cargo_sub "cargo-upgrade"    "upgrade"        "cargo-upgrade"

}
doctor_summary () {

    info_ln '==> Summary \n'

    printf '  ✅ %-18s %s\n' "OK"   "( ${DOCTOR_OK} )"
    printf '  ⚠️ %-18s %s\n' "Warn" "( ${DOCTOR_WARN} )"
    printf '  ❌ %-18s %s\n' "Fail" "( ${DOCTOR_FAIL} )"

    local msg=""

    if (( DOCTOR_FAIL > 0 )); then msg="😡 This isn't a pipeline… it's a crime scene !"
    elif (( DOCTOR_WARN == 0 )); then msg="😎 All is awesome 💯"
    elif (( DOCTOR_WARN == 1 )); then msg="😉 One tiny crack ☕"
    elif (( DOCTOR_WARN == 2 )); then msg="😮‍💨 Two warnings. Still fine ⚠️"
    else msg="😨 Too many warnings !"
    fi

    printf '  %s %-18s %s\n\n' "🤔" "Status" "( ${msg} )"

}

cmd_doctor_help () {

    info_ln "Doctor :\n"

    printf '    %s\n' \
        "doctor                     * Summery of (system + tools + git) full diagnostics" \
        "ensure                     * Ensure all used tools/crates installed" \
        ''

}
cmd_ensure () {

    source <(parse "$@" -- cmd:list)

    info_ln "Ensure Tools ...\n"

    ensure cargo node python git clang jq perl grep curl awk tail sed sort head wc xargs find

    local c=""

    for c in "${cmd[@]:-all}"; do
        [[ "${c}" == "all" || "${c}" == "sanitizer" ]]  && ensure rust-src
        [[ "${c}" == "all" || "${c}" == "miri" ]]       && ensure miri
        [[ "${c}" == "all" || "${c}" == "fmt" ]]        && ensure rustfmt
        [[ "${c}" == "all" || "${c}" == "clippy" ]]     && ensure clippy
        [[ "${c}" == "all" || "${c}" == "taplo" ]]      && ensure taplo
        [[ "${c}" == "all" || "${c}" == "typos" ]]      && ensure typos
        [[ "${c}" == "all" || "${c}" == "syft" ]]       && ensure syft
        [[ "${c}" == "all" || "${c}" == "trivy" ]]      && ensure trivy
        [[ "${c}" == "all" || "${c}" == "leaks" || "${c}" == "gitleaks" ]] && ensure gitleaks
        [[ "${c}" == "all" || "${c}" == "samply" ]]     && ensure samply
        [[ "${c}" == "all" || "${c}" == "flame" ]]      && ensure flamegraph
        [[ "${c}" == "all" || "${c}" == "audit" ]]      && ensure cargo-audit
        [[ "${c}" == "all" || "${c}" == "deny" ]]       && ensure cargo-deny
        [[ "${c}" == "all" || "${c}" == "nextest" ]]    && ensure cargo-nextest
        [[ "${c}" == "all" || "${c}" == "semver" ]]     && ensure cargo-semver-checks
        [[ "${c}" == "all" || "${c}" == "edit" ]]       && ensure cargo-edit
        [[ "${c}" == "all" || "${c}" == "hack" ]]       && ensure cargo-hack
        [[ "${c}" == "all" || "${c}" == "fuzz" ]]       && ensure cargo-fuzz
        [[ "${c}" == "all" || "${c}" == "udeps" ]]      && ensure cargo-udeps
        [[ "${c}" == "all" || "${c}" == "bloat" ]]      && ensure cargo-bloat
        [[ "${c}" == "all" || "${c}" == "vet" ]]        && ensure cargo-vet
        [[ "${c}" == "all" || "${c}" == "cov" ]]        && ensure llvm-tools-preview cargo-llvm-cov
    done

    success_ln "Tools Installed\n"

}
cmd_doctor () {

    DOCTOR_OK=0
    DOCTOR_WARN=0
    DOCTOR_FAIL=0

    doctor_sys
    doctor_github
    doctor_tools
    doctor_rust
    doctor_summary

    (( DOCTOR_FAIL == 0 )) || return 1
    return 0

}
