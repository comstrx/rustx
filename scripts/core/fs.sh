#!/usr/bin/env bash

[[ "${BASH_SOURCE[0]}" != "${0}" ]] || { printf '%s\n' "fs.sh: this file should not be run externally." >&2; exit 2; }
[[ -n "${FS_LOADED:-}" ]] && return 0
FS_LOADED=1

__dir="${BASH_SOURCE[0]%/*}"
[[ "${__dir}" == "${BASH_SOURCE[0]}" ]] && __dir="."
__core_dir="$(cd -- "${__dir}" && pwd -P)"
source "${__core_dir}/pkg.sh"

FS_TMP_PATHS=()

fs_path_expand () {

    local p="${1-}"
    [[ -n "${p}" ]] || { printf '%s' ""; return 0; }

    case "${p}" in
        "~")
            [[ -n "${HOME:-}" ]] || die "HOME not set; cannot expand ~" 2
            p="${HOME}"
        ;;
        "~"/*)
            [[ -n "${HOME:-}" ]] || die "HOME not set; cannot expand ~/" 2
            p="${HOME}/${p#\~/}"
        ;;
    esac

    printf '%s' "${p}"

}
fs_path_dirname () {

    local p=""
    p="$(fs_path_expand "${1-}")"
    [[ -n "${p}" ]] || { printf '%s' "."; return 0; }

    local clean="${p}"

    while [[ "${clean}" != "/" && "${clean}" == */ ]]; do
        clean="${clean%/}"
    done
    [[ -z "${clean//\/}" ]] && clean="/"

    [[ "${clean}" == "/" ]] && { printf '%s' "/"; return 0; }

    if [[ "${clean}" == */* ]]; then

        local d="${clean%/*}"
        [[ -n "${d}" ]] || d="/"

        printf '%s' "${d}"
        return 0

    fi

    printf '%s' "."
    return 0

}
config_file () {

    local name="${1:-}" ext1="${2:-}" ext2="${3:-}"
    local root="${ROOT_DIR:-${PWD}}" base="" candidate=""
    local -a candidates=()

    [[ -n "${name}" ]] || { printf '\n'; return 0; }

    base="${name%%-*}"

    [[ -n "${ext1}" ]] && candidates+=( "${name}.${ext1}" ".${name}.${ext1}" )
    [[ -n "${ext2}" ]] && candidates+=( "${name}.${ext2}" ".${name}.${ext2}" )

    if [[ "${base}" != "${name}" ]]; then
        [[ -n "${ext1}" ]] && candidates+=( "${base}.${ext1}" ".${base}.${ext1}" )
        [[ -n "${ext2}" ]] && candidates+=( "${base}.${ext2}" ".${base}.${ext2}" )
    fi

    for candidate in "${candidates[@]-}"; do
        [[ -f "${root}/${candidate}" ]] || continue
        printf '%s\n' "${root}/${candidate}"
        return 0
    done

    printf '\n'

}
fs_guard_rm_target () {

    local p=""
    p="$(fs_path_expand "${1-}")"
    [[ -n "${p}" ]] || die "fs: refusing empty remove target" 2

    local clean="${p}"

    case "${clean}" in
        *$'\n'*|*$'\r'*)
            die "fs: refusing path with newline characters" 2
        ;;
    esac

    while [[ "${clean}" != "/" && "${clean}" == */ ]]; do
        clean="${clean%/}"
    done
    [[ -z "${clean//\/}" ]] && clean="/"

    case "${clean}" in
        /|.|..|/.|/..)
            die "fs: refusing dangerous remove target: ${clean}" 2
        ;;
    esac

    case "${clean}" in
        *"/."|*"/.."|*"/./"*|*"/../"*)
            die "fs: refusing path with dot segments: ${clean}" 2
        ;;
    esac

    if [[ "${clean}" == ./* ]]; then
        [[ "${FS_ALLOW_RM_DOT:-0}" -eq 1 ]] || die "fs: refusing ./ target; set FS_ALLOW_RM_DOT=1: ${clean}" 2
    fi
    if [[ "${clean}" == ../* ]]; then
        [[ "${FS_ALLOW_RM_DOTDOT:-0}" -eq 1 ]] || die "fs: refusing ../ target; set FS_ALLOW_RM_DOTDOT=1: ${clean}" 2
    fi
    if [[ -n "${HOME:-}" ]]; then

        local home="${HOME%/}"
        if [[ -n "${home}" && "${clean}" == "${home}" ]]; then
            [[ "${FS_ALLOW_RM_HOME:-0}" -eq 1 ]] || die "fs: refusing to remove HOME; set FS_ALLOW_RM_HOME=1: ${clean}" 2
        fi

    fi
    if has is_wsl && is_wsl; then

        if [[ "${clean}" =~ ^/mnt/[a-zA-Z]($|/) ]]; then
            [[ "${FS_ALLOW_RM_MOUNT:-0}" -eq 1 ]] || die "fs: refusing to remove WSL mount; set FS_ALLOW_RM_MOUNT=1: ${clean}" 2
        fi

    fi

    if [[ "$(os_name)" == "windows" ]]; then

        if [[ "${clean}" =~ ^/[a-zA-Z]($|/) || "${clean}" =~ ^/cygdrive/[a-zA-Z]($|/) ]]; then
            [[ "${FS_ALLOW_RM_MOUNT:-0}" -eq 1 ]] || die "fs: refusing to remove Windows drive mount; set FS_ALLOW_RM_MOUNT=1: ${clean}" 2
        fi

    fi

    return 0

}
fs_mkdir_p () {

    local d="${1-}"
    d="$(fs_path_expand "${d}")"
    [[ -n "${d}" ]] || die "fs_mkdir_p: missing dir" 2

    has mkdir || die "fs_mkdir_p: missing required command: mkdir" 2

    command mkdir -p -- "${d}" 2>/dev/null && return 0

    command mkdir -p "${d}" || die "fs_mkdir_p: failed: ${d}" 2

    return 0

}
fs_mv () {

    local src="${1-}"
    local dst="${2-}"

    [[ -n "${src}" && -n "${dst}" ]] || die "fs_mv: usage: fs_mv <src> <dst>" 2
    has mv || die "fs_mv: missing required command: mv" 2

    command mv -f -- "${src}" "${dst}" 2>/dev/null && return 0
    command mv -f "${src}" "${dst}" || die "fs_mv: failed: ${src} -> ${dst}" 2

    return 0

}
fs_rm_dir () {

    local d="${1-}"
    [[ -n "${d}" ]] || die "fs_rm_dir: missing dir" 2
    has rm || die "fs_rm_dir: missing required command: rm" 2

    fs_guard_rm_target "${d}"

    command rm -rf -- "${d}" 2>/dev/null && return 0
    command rm -rf "${d}" || die "fs_rm_dir: failed: ${d}" 2

    return 0

}
fs_tmp_cleanup () {

    local p=""

    (( ${#FS_TMP_PATHS[@]} )) || return 0

    for p in "${FS_TMP_PATHS[@]}"; do

        [[ -n "${p}" ]] || continue

        ( fs_guard_rm_target "${p}" ) >/dev/null 2>&1 || continue

        command rm -rf -- "${p}" 2>/dev/null || true

    done

    FS_TMP_PATHS=()
    return 0

}
fs_tmp_arm () {

    (( ${#FS_TMP_PATHS[@]} )) && return 0

    trap 'fs_tmp_cleanup' EXIT
    trap 'fs_tmp_cleanup; trap - INT; kill -INT "$$"' INT
    trap 'fs_tmp_cleanup; trap - TERM; kill -TERM "$$"' TERM
    trap 'fs_tmp_cleanup; trap - HUP; kill -HUP "$$"' HUP

    return 0

}
fs_tmp_track () {

    local p="${1-}"
    [[ -n "${p}" ]] || return 0

    fs_tmp_arm
    FS_TMP_PATHS+=( "${p}" )

    return 0

}
fs_tmp_release () {

    local p="${1-}"
    [[ -n "${p}" ]] || return 0

    local x=""
    local -a keep=()

    for x in "${FS_TMP_PATHS[@]-}"; do
        [[ -n "${x}" ]] || continue
        [[ "${x}" == "${p}" ]] && continue
        keep+=( "${x}" )
    done

    if (( ${#keep[@]} )); then FS_TMP_PATHS=( "${keep[@]}" )
    else FS_TMP_PATHS=()
    fi

    ( fs_guard_rm_target "${p}" ) >/dev/null 2>&1 || return 0
    command rm -rf -- "${p}" 2>/dev/null || true

    return 0

}
fs_tmp_dir () {

    local __fs_out_ref="${1-}"
    local __fs_prefix="${2:-rustx}"

    [[ -n "${__fs_out_ref}" ]] || die "fs_tmp_dir: usage: fs_tmp_dir <out-var> [prefix]" 2
    has mktemp || die "fs_tmp_dir: missing required command: mktemp" 2

    local __fs_base="${TMPDIR:-/tmp}" __fs_path=""

    __fs_base="${__fs_base%/}"
    [[ -n "${__fs_base}" ]] || __fs_base="/tmp"

    __fs_path="$(command mktemp -d "${__fs_base}/${__fs_prefix}.XXXXXXXXXX" 2>/dev/null || true)"
    [[ -n "${__fs_path}" && -d "${__fs_path}" ]] || die "fs_tmp_dir: failed to create temporary directory" 2

    command chmod 700 -- "${__fs_path}" 2>/dev/null || command chmod 700 "${__fs_path}" 2>/dev/null || true

    fs_tmp_track "${__fs_path}"

    local -n __fs_out="${__fs_out_ref}"
    __fs_out="${__fs_path}"

    return 0

}
fs_tmp_file () {

    local __fs_out_ref="${1-}"
    local __fs_dir="${2-}"
    local __fs_prefix="${3:-rustx}"

    [[ -n "${__fs_out_ref}" ]] || die "fs_tmp_file: usage: fs_tmp_file <out-var> [dir] [prefix]" 2
    has mktemp || die "fs_tmp_file: missing required command: mktemp" 2

    local __fs_path="" __fs_base="${TMPDIR:-/tmp}"

    __fs_base="${__fs_base%/}"
    [[ -n "${__fs_base}" ]] || __fs_base="/tmp"

    if [[ -n "${__fs_dir}" && -d "${__fs_dir}" ]]; then
        __fs_path="$(command mktemp "${__fs_dir%/}/.${__fs_prefix}.XXXXXXXX" 2>/dev/null || true)"
    fi

    [[ -n "${__fs_path}" ]] || __fs_path="$(command mktemp "${__fs_base}/${__fs_prefix}.XXXXXXXX" 2>/dev/null || true)"
    [[ -n "${__fs_path}" && -f "${__fs_path}" ]] || die "fs_tmp_file: mktemp failed" 2

    fs_tmp_track "${__fs_path}"

    local -n __fs_out="${__fs_out_ref}"
    __fs_out="${__fs_path}"

    return 0

}
new_file () {

    local f="${1-}"
    [[ -n "${f}" ]] || die "new_file: missing file path" 2

    ensure_file "${f}"

}
remove_dir () {

    local d="${1-}"
    [[ -n "${d}" ]] || die "remove_dir: missing dir" 2

    d="$(fs_path_expand "${d}")"

    fs_guard_rm_target "${d}"

    [[ -e "${d}" ]] || return 0
    [[ -d "${d}" ]] || die "remove_dir: not a dir: ${d}" 2

    fs_rm_dir "${d}"

}
move_file () {

    local src="${1-}"
    local dst="${2-}"

    [[ -n "${src}" && -n "${dst}" ]] || die "move_file: usage: move_file <src> <dst>" 2

    src="$(fs_path_expand "${src}")"
    dst="$(fs_path_expand "${dst}")"

    [[ -f "${src}" ]] || die "move_file: missing source file: ${src}" 2

    fs_mv "${src}" "${dst}"

}
ensure_dir () {

    local path="" mode="" owner="" group="" strict=0

    path="$(fs_path_expand "${1:-}")"
    [[ -n "${path}" ]] || die "ensure_dir: missing path" 2

    mode="${2:-}"
    owner="${3:-}"
    group="${4:-}"
    strict="${5:-0}"

    if [[ -e "${path}" && ! -d "${path}" ]]; then
        die "ensure_dir: path exists but not a directory: ${path}" 2
    fi

    [[ -d "${path}" ]] || fs_mkdir_p "${path}"

    if [[ -n "${mode}" ]]; then
        if has chmod; then
            command chmod "${mode}" -- "${path}" 2>/dev/null || command chmod "${mode}" "${path}" 2>/dev/null || {
                if (( strict )); then
                    die "ensure_dir: chmod failed for ${path}" 2
                    return $?
                fi
                true
            }
        else
            (( strict )) && die "ensure_dir: chmod not available for ${path}" 2
        fi
    fi

    if [[ -n "${owner}" ]]; then
        if has chown; then
            if [[ -n "${group}" ]]; then
                command chown "${owner}:${group}" -- "${path}" 2>/dev/null || command chown "${owner}:${group}" "${path}" 2>/dev/null || {
                    if (( strict )); then
                        die "ensure_dir: chown failed for ${path}" 2
                        return $?
                    fi
                    true
                }
            else
                command chown "${owner}" -- "${path}" 2>/dev/null || command chown "${owner}" "${path}" 2>/dev/null || {
                    if (( strict )); then
                        die "ensure_dir: chown failed for ${path}" 2
                        return $?
                    fi
                    true
                }
            fi
        else
            (( strict )) && die "ensure_dir: chown not available for ${path}" 2
        fi
    fi

    [[ -d "${path}" ]] || die "ensure_dir: directory still missing after create: ${path}" 2

}
ensure_file () {

    local path="" mode="" owner="" group="" strict=0

    path="$(fs_path_expand "${1:-}")"
    [[ -n "${path}" ]] || die "ensure_file: missing path" 2

    mode="${2:-}"
    owner="${3:-}"
    group="${4:-}"
    strict="${5:-0}"

    if [[ -e "${path}" && ! -f "${path}" ]]; then
        die "ensure_file: path exists but not a regular file: ${path}" 2
    fi

    if [[ ! -f "${path}" ]]; then

        ensure_dir "$(fs_path_dirname "${path}")" "" "" "" "${strict}"
        : > "${path}" 2>/dev/null || die "ensure_file: failed to create file: ${path}" 2

    fi

    if [[ -n "${mode}" ]]; then
        if has chmod; then
            command chmod "${mode}" -- "${path}" 2>/dev/null || command chmod "${mode}" "${path}" 2>/dev/null || {
                if (( strict )); then
                    die "ensure_file: chmod failed for ${path}" 2
                    return $?
                fi
                true
            }
        else
            (( strict )) && die "ensure_file: chmod not available for ${path}" 2
        fi
    fi

    if [[ -n "${owner}" ]]; then
        if has chown; then
            if [[ -n "${group}" ]]; then
                command chown "${owner}:${group}" -- "${path}" 2>/dev/null || command chown "${owner}:${group}" "${path}" 2>/dev/null || {
                    if (( strict )); then
                        die "ensure_file: chown failed for ${path}" 2
                        return $?
                    fi
                    true
                }
            else
                command chown "${owner}" -- "${path}" 2>/dev/null || command chown "${owner}" "${path}" 2>/dev/null || {
                    if (( strict )); then
                        die "ensure_file: chown failed for ${path}" 2
                        return $?
                    fi
                    true
                }
            fi
        else
            (( strict )) && die "ensure_file: chown not available for ${path}" 2
        fi
    fi

    [[ -f "${path}" ]] || die "ensure_file: file still missing after create: ${path}" 2

}
file_size () {

    local f="${1-}"
    [[ -n "${f}" ]] || die "file_size: missing file" 2
    [[ -f "${f}" ]] || die "file_size: not a file: ${f}" 2

    local n=""

    if has stat; then

        n="$(command stat -c '%s' -- "${f}" 2>/dev/null || command stat -c '%s' "${f}" 2>/dev/null || true)"
        [[ "${n}" =~ ^[0-9]+$ ]] || n="$(command stat -f '%z' -- "${f}" 2>/dev/null || command stat -f '%z' "${f}" 2>/dev/null || true)"
        [[ "${n}" =~ ^[0-9]+$ ]] && { printf '%s' "${n}"; return 0; }

    fi

    has wc || die "file_size: missing required command: wc" 2

    n="$(command wc -c < "${f}" | tr -d '[:space:]')" || die "file_size: failed: ${f}" 2
    [[ "${n}" =~ ^[0-9]+$ ]] || die "file_size: unreadable size for: ${f}" 2

    printf '%s' "${n}"

}
