#!/usr/bin/env bash

[[ "${BASH_SOURCE[0]}" != "${0}" ]] || { printf '%s\n' "pkg.sh: this file should not be run externally." >&2; exit 2; }
[[ -n "${PKG_LOADED:-}" ]] && return 0
PKG_LOADED=1

__dir="${BASH_SOURCE[0]%/*}"
[[ "${__dir}" == "${BASH_SOURCE[0]}" ]] && __dir="."
__core_dir="$(cd -- "${__dir}" && pwd -P)"
source "${__core_dir}/parse.sh"

pkg_has_cmd () {

    has "${1-}"

}
pkg_hash_clear () {

    hash -r 2>/dev/null || true

}
pkg_path_prepend () {

    local d="${1-}"
    [[ -n "${d}" && -d "${d}" ]] || return 0

    case ":${PATH-}:" in
        *":${d}:"*) ;;
        *) PATH="${d}:${PATH-}" ;;
    esac

    export PATH
    return 0

}
pkg_with_sudo () {

    local uid="${EUID:-}"
    [[ -n "${uid}" ]] || uid="$(id -u 2>/dev/null || printf '%s' 1)"

    if [[ "${uid}" -eq 0 ]]; then
        "$@"
        return $?
    fi

    case "$(os_name)" in
        windows) "$@"; return $? ;;
    esac

    local non_interactive=0

    if declare -F is_ci >/dev/null 2>&1; then

        is_ci && non_interactive=1

    fi
    if pkg_has_cmd sudo; then

        if (( non_interactive )); then sudo -n "$@"
        else sudo "$@"
        fi

        return $?

    fi
    if pkg_has_cmd doas; then

        if (( non_interactive )); then doas -n "$@"
        else doas "$@"
        fi

        return $?

    fi

    die "Need root privileges (sudo/doas not found). Run as root or install sudo/doas." 2

}
pkg_apt_update_once () {

    (( ${PKG_APT_UPDATED:-0} )) && return 0
    PKG_APT_UPDATED=1

    pkg_with_sudo apt-get update >/dev/null 2>&1 || pkg_with_sudo apt-get update

}
pkg_linux_mgr () {

    if pkg_has_cmd apt-get; then printf '%s' apt; return 0; fi
    if pkg_has_cmd dnf; then printf '%s' dnf; return 0; fi
    if pkg_has_cmd yum; then printf '%s' yum; return 0; fi
    if pkg_has_cmd pacman; then printf '%s' pacman; return 0; fi
    if pkg_has_cmd zypper; then printf '%s' zypper; return 0; fi
    if pkg_has_cmd apk; then printf '%s' apk; return 0; fi

    printf '%s' ""
    return 1

}
pkg_linux_install () {

    local mgr="${1-}"
    shift || true

    local -a pkgs=( "$@" )
    (( ${#pkgs[@]} )) || return 0

    local yes=0
    (( YES_ENV )) && yes=1

    case "${mgr}" in
        apt)
            pkg_apt_update_once

            if (( yes )); then pkg_with_sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends "${pkgs[@]}"
            else pkg_with_sudo env DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends "${pkgs[@]}"
            fi
        ;;
        dnf)
            if (( yes )); then pkg_with_sudo dnf install -y "${pkgs[@]}"
            else pkg_with_sudo dnf install "${pkgs[@]}"
            fi
        ;;
        yum)
            if (( yes )); then pkg_with_sudo yum install -y "${pkgs[@]}"
            else pkg_with_sudo yum install "${pkgs[@]}"
            fi
        ;;
        pacman)
            if (( yes )); then pkg_with_sudo pacman -S --needed --noconfirm "${pkgs[@]}"
            else pkg_with_sudo pacman -S --needed "${pkgs[@]}"
            fi
        ;;
        zypper)
            if (( yes )); then pkg_with_sudo zypper --non-interactive install --no-recommends "${pkgs[@]}"
            else pkg_with_sudo zypper install --no-recommends "${pkgs[@]}"
            fi
        ;;
        apk)
            pkg_with_sudo apk add --no-progress "${pkgs[@]}"
        ;;
        *)
            die "Linux: no supported package manager found (apt/dnf/yum/pacman/zypper/apk)." 2
        ;;
    esac

    return 0

}
pkg_mac_install () {

    local -a pkgs=( "$@" )
    (( ${#pkgs[@]} )) || return 0

    pkg_has_cmd brew || die "macOS: Homebrew not found (install brew or preinstall required tools)." 2

    local -a envs=( "HOMEBREW_NO_AUTO_UPDATE=1" "HOMEBREW_NO_INSTALL_CLEANUP=1" )
    env "${envs[@]}" brew install "${pkgs[@]}"

    return 0

}
pkg_win_install () {

    local -a pkgs=( "$@" )
    (( ${#pkgs[@]} )) || return 0

    pkg_has_cmd pacman || die "windows: pacman not found (use MSYS2, or run under WSL/Linux)." 2

    local yes=0
    (( YES_ENV )) && yes=1

    if (( yes )); then run pacman -S --needed --noconfirm "${pkgs[@]}"
    else run pacman -S --needed "${pkgs[@]}"
    fi

    return 0

}
pkg_map_one () {

    local os="$(os_name)"
    local want="${1:-}"
    [[ -n "${want}" ]] || { printf '%s' ""; return 0; }

    case "${os}" in
        linux)
            local mgr="$(pkg_linux_mgr 2>/dev/null || true)"
            case "${want}" in
                python|python3|python3.*) printf '%s' "python3" ; return 0 ;;
                pip|pip3)
                    case "${mgr}" in
                        pacman) printf '%s' "python-pip" ;;
                        apk)    printf '%s' "py3-pip" ;;
                        *)      printf '%s' "python3-pip" ;;
                    esac
                    return 0
                ;;
                awk)        printf '%s' "gawk" ; return 0 ;;
                find|xargs)  printf '%s' "findutils" ; return 0 ;;
                grep|sed|git|perl|jq|curl) printf '%s' "${want}" ; return 0 ;;
                mv|cp|rm|ln|mkdir|rmdir|cat|touch|head|tail|cut|tr|sort|uniq|wc|date|sleep|mktemp|basename|dirname|realpath|tee|chmod|readlink|stat)
                    printf '%s' "coreutils"
                    return 0
                ;;
                llvm-dev|llvm-devel|llvm-config)
                    case "${mgr}" in
                        apt)           printf '%s' "llvm-dev" ;;
                        dnf|yum|zypper) printf '%s' "llvm-devel" ;;
                        pacman)        printf '%s' "llvm" ;;
                        apk)           printf '%s' "llvm-dev" ;;
                        *)             printf '%s' "${want}" ;;
                    esac
                    return 0
                ;;
                libclang-dev|libclang-devel)
                    case "${mgr}" in
                        apt)           printf '%s' "libclang-dev" ;;
                        dnf|yum|zypper) printf '%s' "libclang-devel" ;;
                        pacman)        printf '%s' "libclang" ;;
                        apk)           printf '%s' "clang-dev" ;;
                        *)             printf '%s' "${want}" ;;
                    esac
                    return 0
                ;;
            esac
        ;;
        mac)
            case "${want}" in
                python|python3|python3.*) printf '%s' "python" ; return 0 ;;
                pip|pip3)                 printf '%s' "python" ; return 0 ;;
                awk)        printf '%s' "gawk" ; return 0 ;;
                sed)        printf '%s' "gnu-sed" ; return 0 ;;
                grep)       printf '%s' "grep" ; return 0 ;;
                find|xargs)  printf '%s' "findutils" ; return 0 ;;
                head|tail|sort|wc|chmod|mkdir|date|readlink|realpath|stat)
                    printf '%s' "coreutils"
                    return 0
                ;;
                git|jq)
                    printf '%s' "${want}"
                    return 0
                ;;
                perl|curl)
                    pkg_has_cmd "${want}" && { printf '%s' ""; return 0; }
                    printf '%s' "${want}"
                    return 0
                ;;
                mktemp|mv|tr|tee)
                    pkg_has_cmd "${want}" && { printf '%s' ""; return 0; }
                    printf '%s' "coreutils"
                    return 0
                ;;
                llvm-dev|llvm-devel|llvm-config|libclang-dev|libclang-devel|clang-dev|llvm|libclang|clang)
                    printf '%s' "llvm"
                    return 0
                ;;
            esac
        ;;
        windows)
            case "${want}" in
                python|python3|python3.*) printf '%s' "python" ; return 0 ;;
                pip|pip3)                 printf '%s' "python-pip" ; return 0 ;;
                awk)        printf '%s' "gawk" ; return 0 ;;
                find|xargs)  printf '%s' "findutils" ; return 0 ;;
                sed|grep|git|perl|jq|curl) printf '%s' "${want}" ; return 0 ;;
                mv|cp|rm|ln|mkdir|rmdir|cat|touch|head|tail|cut|tr|sort|uniq|wc|date|sleep|mktemp|basename|dirname|realpath|tee|chmod|readlink|stat)
                    printf '%s' "coreutils"
                    return 0
                ;;
                llvm-dev|llvm-devel|llvm-config|libclang-dev|libclang-devel|clang-dev|llvm|libclang|clang)
                    printf '%s' "llvm"
                    return 0
                ;;
            esac
        ;;
    esac

    printf '%s' "${want}"
    return 0

}
pkg_map_list () {

    local -n out_pkgs="${1}"
    shift || true

    local want="" p=""
    out_pkgs=()

    for want in "$@"; do

        [[ -n "${want}" ]] || continue

        p="$(pkg_map_one "${want}")"
        [[ -n "${p}" ]] || continue

        out_pkgs+=( "${p}" )

    done

    return 0

}
pkg_uniq_list () {

    local -n out_uniq="${1}"
    shift || true

    local x="" y="" found=0
    out_uniq=()

    for x in "$@"; do

        [[ -n "${x}" ]] || continue
        found=0

        for y in "${out_uniq[@]-}"; do

            if [[ "${y}" == "${x}" ]]; then
                found=1
                break
            fi

        done

        (( found )) && continue
        out_uniq+=( "${x}" )

    done

    return 0

}
pkg_mac_link_cmd () {

    local alt="${1-}"
    local name="${2-}"

    [[ -n "${alt}" && -n "${name}" ]] || return 0
    pkg_has_cmd "${alt}" || return 0

    local bin_dir="${HOME}/.local/bin"
    local link_path="${bin_dir}/${name}"

    local alt_path="$(command -v -- "${alt}" 2>/dev/null || true)"
    [[ -n "${alt_path}" ]] || return 0

    if [[ -L "${link_path}" ]]; then
        local cur="$(readlink "${link_path}" 2>/dev/null || true)"
        [[ "${cur}" == "${alt_path}" ]] && return 0
    fi

    run mkdir -p -- "${bin_dir}" 2>/dev/null || true
    run rm -f -- "${link_path}" 2>/dev/null || true
    run ln -s -- "${alt_path}" "${link_path}" 2>/dev/null || true

    return 0

}
pkg_mac_gnu_shim () {

    local -a wants=( "$@" )
    (( ${#wants[@]} )) || return 0

    local bin_dir="${HOME}/.local/bin" need_llvm=0 w=""

    run mkdir -p -- "${bin_dir}" 2>/dev/null || true
    pkg_path_prepend "${bin_dir}"

    for w in "${wants[@]}"; do
        case "${w}" in
            clang|llvm-config|llvm-dev|llvm-devel|libclang-dev|libclang-devel|clang-dev|llvm|libclang) need_llvm=1 ;;
        esac
    done

    if (( need_llvm )) && pkg_has_cmd brew; then
        if brew list --versions llvm >/dev/null 2>&1; then
            local llvm_prefix="$(brew --prefix llvm 2>/dev/null || true)"
            [[ -n "${llvm_prefix}" && -d "${llvm_prefix}/bin" ]] && pkg_path_prepend "${llvm_prefix}/bin"
        fi
    fi

    local -a uniq_wants=()
    pkg_uniq_list uniq_wants "${wants[@]}"

    for w in "${uniq_wants[@]}"; do
        case "${w}" in
            awk)      pkg_mac_link_cmd gawk      awk ;;
            sed)      pkg_mac_link_cmd gsed      sed ;;
            grep)     pkg_mac_link_cmd ggrep     grep ;;
            find)     pkg_mac_link_cmd gfind     find ;;
            xargs)    pkg_mac_link_cmd gxargs    xargs ;;
            head)     pkg_mac_link_cmd ghead     head ;;
            tail)     pkg_mac_link_cmd gtail     tail ;;
            sort)     pkg_mac_link_cmd gsort     sort ;;
            wc)       pkg_mac_link_cmd gwc       wc ;;
            chmod)    pkg_mac_link_cmd gchmod    chmod ;;
            mkdir)    pkg_mac_link_cmd gmkdir    mkdir ;;
            date)     pkg_mac_link_cmd gdate     date ;;
            stat)     pkg_mac_link_cmd gstat     stat ;;
            readlink) pkg_mac_link_cmd greadlink readlink ;;
            realpath) pkg_mac_link_cmd grealpath realpath ;;
            tr)       pkg_mac_link_cmd gtr       tr ;;
            tee)      pkg_mac_link_cmd gtee      tee ;;
            mktemp)   pkg_mac_link_cmd gmktemp   mktemp ;;
        esac
    done

    pkg_hash_clear
    return 0

}
pkg_want_is_pkg_only () {

    local want="${1:-}"
    [[ -n "${want}" ]] || return 1

    case "${want}" in
        llvm-dev|llvm-devel|libclang-dev|libclang-devel|clang-dev|llvm|libclang) return 0 ;;
    esac

    return 1

}
pkg_verify_wants () {

    local os="$(os_name)" w="" p=""
    local -a wants=( "$@" )

    for w in "${wants[@]}"; do

        [[ -n "${w}" ]] || continue

        pkg_want_is_pkg_only "${w}" && continue

        if [[ "${w}" == "python" ]]; then

            if [[ "${os}" == "windows" ]]; then pkg_has_cmd python || die "pkg: missing command 'python' after install" 2
            else pkg_has_cmd python3 || die "pkg: missing command 'python3' after install" 2
            fi

            continue

        fi
        if [[ "${w}" == "pip" || "${w}" == "pip3" ]]; then

            pkg_has_cmd pip || pkg_has_cmd pip3 || die "pkg: missing pip after install" 2
            continue

        fi
        if [[ "${os}" == "mac" ]]; then

            case "${w}" in
                awk|sed|grep|find|xargs|head|tail|sort|wc|chmod|mkdir|date|stat|readlink|realpath)
                    p="$(command -v -- "${w}" 2>/dev/null || true)"
                    [[ "${p}" == "${HOME}/.local/bin/${w}" ]] || die "pkg: command '${w}' is not GNU-shimmed (expected ${HOME}/.local/bin/${w})" 2
                ;;
            esac

        fi

    done

    return 0

}
pkg_collect_missing_wants () {

    local -n out_missing="${1}"
    shift || true

    local os="$(os_name)" w=""
    out_missing=()

    for w in "$@"; do

        [[ -n "${w}" ]] || continue
        pkg_want_is_pkg_only "${w}" && continue

        if [[ "${w}" == "python" ]]; then

            if [[ "${os}" == "windows" ]]; then pkg_has_cmd python || out_missing+=( "python" )
            else pkg_has_cmd python3 || out_missing+=( "python3" )
            fi

            continue

        fi
        if [[ "${w}" == "pip" || "${w}" == "pip3" ]]; then

            pkg_has_cmd pip || pkg_has_cmd pip3 || out_missing+=( "pip" )
            continue

        fi

        pkg_has_cmd "${w}" || out_missing+=( "${w}" )

    done

    return 0

}
pkg_install_linux () {

    local -a pkgs=( "$@" )
    local mgr="$(pkg_linux_mgr)" || true

    (( ${#pkgs[@]} )) || die "pkg_install_linux: missing package name(s)" 2
    [[ -n "${mgr}" ]] || die "Linux: no supported package manager found (apt/dnf/yum/pacman/zypper/apk)." 2

    pkg_linux_install "${mgr}" "${pkgs[@]}"

}
pkg_install_mac () {

    local -a pkgs=( "$@" )
    (( ${#pkgs[@]} )) || die "pkg_install_mac: missing package name(s)" 2

    pkg_mac_install "${pkgs[@]}"

}
pkg_install_win () {

    local -a pkgs=( "$@" )
    (( ${#pkgs[@]} )) || die "pkg_install_win: missing package name(s)" 2

    declare -F is_wsl >/dev/null 2>&1 && is_wsl && die "WSL detected: use pkg_install_linux instead of pkg_install_win" 2
    pkg_win_install "${pkgs[@]}"

}
pkg_is_installed () {

    local os="${1-}" p="${2-}"
    [[ -n "${os}" && -n "${p}" ]] || return 1

    case "${os}" in
        linux)
            if pkg_has_cmd dpkg; then dpkg -s "${p}" >/dev/null 2>&1; return $?; fi
            if pkg_has_cmd rpm;  then rpm -q "${p}" >/dev/null 2>&1; return $?; fi
            if pkg_has_cmd pacman; then pacman -Qi "${p}" >/dev/null 2>&1; return $?; fi
            if pkg_has_cmd apk; then apk info -e "${p}" >/dev/null 2>&1; return $?; fi
            return 1
        ;;
        mac)
            pkg_has_cmd brew || return 1
            brew list --versions "${p}" >/dev/null 2>&1
            return $?
        ;;
        windows)
            pkg_has_cmd pacman || return 1
            pacman -Qi "${p}" >/dev/null 2>&1
            return $?
        ;;
    esac

    return 1

}
ensure_pkg () {

    local yes=0 quiet=0 verbose=0 p="" os="$(os_name)"

    local -a pkgs=() uniq_pkgs=()
    local -a wants=()

    source <(parse "$@" -- --yes:bool --quiet:bool --verbose:bool :wants:list)

    (( YES_ENV || yes )) && YES_ENV=1
    (( QUIET_ENV || quiet )) && QUIET_ENV=1
    (( VERBOSE_ENV || verbose )) && VERBOSE_ENV=1

    pkg_map_list pkgs "${wants[@]}"
    pkg_uniq_list uniq_pkgs "${pkgs[@]}"

    (( ${#uniq_pkgs[@]} )) || { pkg_hash_clear; pkg_verify_wants "${wants[@]}"; return 0; }

    local -A _pkg_ok=()
    local -a missing_pkgs=()

    for p in "${uniq_pkgs[@]}"; do

        [[ -n "${p}" ]] || continue
        [[ -n "${_pkg_ok[${p}]-}" ]] && continue

        if pkg_is_installed "${os}" "${p}"; then _pkg_ok["${p}"]=1
        else missing_pkgs+=( "${p}" )
        fi

    done

    if (( ${#missing_pkgs[@]} == 0 )); then

        is_mac && pkg_mac_gnu_shim "${wants[@]}"
        pkg_hash_clear
        pkg_verify_wants "${wants[@]}"

        return 0

    fi

    case "${os}" in
        linux)
            pkg_install_linux "${missing_pkgs[@]}"
        ;;
        mac)
            pkg_install_mac "${missing_pkgs[@]}"
            pkg_mac_gnu_shim "${wants[@]}"
        ;;
        windows)
            if pkg_has_cmd pacman; then
                pkg_install_win "${missing_pkgs[@]}"
            else
                local -a missing=()
                pkg_collect_missing_wants missing "${wants[@]}"
                (( ${#missing[@]} )) && die "windows: missing commands (${missing[*]}). run under WSL/Linux." 2
            fi
        ;;
        *)
            die "ensure_pkg: unsupported OS '${os}'" 2
        ;;
    esac

    pkg_hash_clear
    pkg_verify_wants "${wants[@]}"

    return 0

}
