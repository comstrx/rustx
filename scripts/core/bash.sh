#!/usr/bin/env bash

bash_die () {

    local msg="${1-}" code="${2:-2}"
    printf '%s\n' "${msg:-ensure-bash: failed}" >&2
    exit "${code}"

}
bash_major_from_bin () {

    local bash_bin="${1-}" major=""
    [[ -n "${bash_bin}" && -x "${bash_bin}" ]] || { printf '0'; return 0; }

    major="$("${bash_bin}" -c 'printf "%s" "${BASH_VERSINFO[0]:-0}"' 2>/dev/null || true)"

    case "${major}" in
        ""|*[!0-9]*) printf '0' ;;
        *)           printf '%s' "${major}" ;;
    esac

}
bash_prefer_bin () {

    local bash_bin="${1-}" want_major="${2:-5}" dir=""

    [[ -n "${bash_bin}" && -x "${bash_bin}" ]] || return 1
    (( $(bash_major_from_bin "${bash_bin}") >= want_major )) || return 1

    dir="${bash_bin%/*}"
    [[ -n "${dir}" && "${dir}" != "${bash_bin}" ]] || return 0

    PATH="${dir}:${PATH}"
    export PATH

    return 0

}
bash_sudo () {

    if (( EUID == 0 )); then
        "$@"
        return $?
    fi

    command -v sudo >/dev/null 2>&1 || return 127

    if [[ -n "${CI:-}" ]]; then
        sudo -n "$@"
        return $?
    fi

    sudo "$@"

}
ensure_linux_bash () {

    local want_major="${1:-5}"

    bash_prefer_bin "$(command -v bash 2>/dev/null || true)" "${want_major}" && return 0

    if command -v apt-get >/dev/null 2>&1; then

        bash_sudo apt-get update || true
        bash_sudo apt-get install -y bash || return 1

    elif command -v apt >/dev/null 2>&1; then

        bash_sudo apt update || true
        bash_sudo apt install -y bash || return 1

    elif command -v dnf >/dev/null 2>&1; then

        bash_sudo dnf install -y bash || return 1

    elif command -v yum >/dev/null 2>&1; then

        bash_sudo yum install -y bash || return 1

    elif command -v pacman >/dev/null 2>&1; then

        bash_sudo pacman -S --noconfirm bash || return 1

    elif command -v zypper >/dev/null 2>&1; then

        bash_sudo zypper --non-interactive install bash || return 1

    elif command -v apk >/dev/null 2>&1; then

        bash_sudo apk add --no-cache bash || return 1

    else

        return 1

    fi

    bash_prefer_bin "$(command -v bash 2>/dev/null || true)" "${want_major}"

}
ensure_mac_bash () {

    local want_major="${1:-5}"
    local cand="" prefix=""

    bash_prefer_bin "$(command -v bash 2>/dev/null || true)" "${want_major}" && return 0

    for cand in /opt/homebrew/bin/bash /usr/local/bin/bash; do
        bash_prefer_bin "${cand}" "${want_major}" && return 0
    done

    command -v brew >/dev/null 2>&1 || return 1

    prefix="$(brew --prefix bash 2>/dev/null || true)"
    [[ -n "${prefix}" ]] && bash_prefer_bin "${prefix}/bin/bash" "${want_major}" && return 0

    brew install bash >/dev/null 2>&1 || return 1

    prefix="$(brew --prefix bash 2>/dev/null || brew --prefix 2>/dev/null || true)"
    [[ -n "${prefix}" ]] || return 1

    bash_prefer_bin "${prefix}/bin/bash" "${want_major}"

}
ensure_win_bash () {

    local want_major="${1:-5}"

    if [[ -n "${WSL_DISTRO_NAME:-}" ]] || grep -qi microsoft /proc/version 2>/dev/null; then
        ensure_linux_bash "${want_major}"
        return $?
    fi

    bash_prefer_bin "$(command -v bash 2>/dev/null || true)" "${want_major}" && return 0

    command -v pacman >/dev/null 2>&1 || return 1
    pacman -S --needed --noconfirm bash >/dev/null 2>&1 || return 1

    bash_prefer_bin "$(command -v bash 2>/dev/null || true)" "${want_major}"

}
ensure_bash () {

    local want_major="${1:-5}"
    shift 1 || true

    local cur_major="${BASH_VERSINFO[0]:-0}"
    (( cur_major >= want_major )) && return 0

    [[ -n "${BASH_BOOTSTRAPPED:-}" ]] && bash_die "ensure-bash: requires bash >= ${want_major}" 2

    local uname_s=""
    uname_s="$(uname -s 2>/dev/null | tr '[:upper:]' '[:lower:]' || true)"
    [[ -n "${uname_s}" ]] || uname_s="${OSTYPE:-}"

    case "${uname_s}" in
        linux*) ensure_linux_bash "${want_major}" || bash_die "ensure-bash: install/upgrade bash ${want_major}+ on Linux failed" 2 ;;
        darwin*) ensure_mac_bash "${want_major}" || bash_die "ensure-bash: install Homebrew bash ${want_major}+ on macOS failed (need brew)" 2 ;;
        msys*|mingw*|cygwin*) ensure_win_bash "${want_major}" || bash_die "ensure-bash: update Git Bash/MSYS2 to bash ${want_major}+ (MSYS2: pacman -S bash)" 2 ;;
        *) bash_die "ensure-bash: unsupported OS '${uname_s}'" 2 ;;
    esac

    local bash_bin="" new_major=""

    bash_bin="$(command -v bash 2>/dev/null || true)"
    new_major="$(bash_major_from_bin "${bash_bin}")"

    (( new_major >= want_major )) || bash_die "ensure-bash: bash ${want_major}+ not available" 2

    export BASH_BOOTSTRAPPED=1
    exec "${bash_bin}" "$0" "$@" || bash_die "ensure-bash: failed to execute bash v${want_major}"

}

ensure_bash 5 "$@"
