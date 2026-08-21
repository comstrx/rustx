#!/usr/bin/env bash

[[ "${BASH_SOURCE[0]}" != "${0}" ]] || { printf '%s\n' "fs.sh: this file should not be run externally." >&2; exit 2; }
[[ -n "${FS_LOADED:-}" ]] && return 0
FS_LOADED=1

__dir="${BASH_SOURCE[0]%/*}"
[[ "${__dir}" == "${BASH_SOURCE[0]}" ]] && __dir="."
__core_dir="$(cd -- "${__dir}" && pwd -P)"
source "${__core_dir}/pkg.sh"

fs_path_expand () {

    local p="${1-}"
    [[ -n "${p}" ]] || { printf '%s' ""; return 0; }

    case "${p}" in
        "~")
            [[ -n "${HOME:-}" ]] || die "HOME not set; cannot expand ~" 2
            p="${HOME}"
        ;;
        "~/"*)
            [[ -n "${HOME:-}" ]] || die "HOME not set; cannot expand ~/" 2
            p="${HOME}/${p#\~/}"
        ;;
    esac

    printf '%s' "${p}"

}
fs_path_basename () {

    local p=""
    p="$(fs_path_expand "${1-}")"
    [[ -n "${p}" ]] || { printf '%s' ""; return 0; }

    local clean="${p}"

    while [[ "${clean}" != "/" && "${clean}" == */ ]]; do
        clean="${clean%/}"
    done
    [[ -z "${clean//\/}" ]] && clean="/"

    [[ "${clean}" == "/" ]] && { printf '%s' "/"; return 0; }

    printf '%s' "${clean##*/}"

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

    case "$(uname -s 2>/dev/null || true)" in
        MINGW*|MSYS*|CYGWIN*)
            if [[ "${clean}" =~ ^/[a-zA-Z]($|/) ]]; then
                [[ "${FS_ALLOW_RM_MOUNT:-0}" -eq 1 ]] || die "fs: refusing to remove Windows drive mount; set FS_ALLOW_RM_MOUNT=1: ${clean}" 2
            fi
        ;;
    esac

    return 0

}
fs_mkdir_p () {

    local d="${1-}"
    d="$(fs_path_expand "${d}")"
    [[ -n "${d}" ]] || die "fs_mkdir_p: missing dir" 2

    has mkdir || die "fs_mkdir_p: missing required command: mkdir" 2

    command mkdir -p -- "${d}" 2>/dev/null && return 0
    command mkdir -p "${d}" 2>/dev/null && return 0

    die "fs_mkdir_p: failed: ${d}" 2

}
fs_mv () {

    local src="${1-}"
    local dst="${2-}"

    [[ -n "${src}" && -n "${dst}" ]] || die "fs_mv: usage: fs_mv <src> <dst>" 2
    has mv || die "fs_mv: missing required command: mv" 2

    command mv -f -- "${src}" "${dst}" 2>/dev/null && return 0
    command mv -f "${src}" "${dst}" 2>/dev/null && return 0

    die "fs_mv: failed: ${src} -> ${dst}" 2

}
fs_cp_file () {

    local src="${1-}"
    local dst="${2-}"

    [[ -n "${src}" && -n "${dst}" ]] || die "fs_cp_file: usage: fs_cp_file <src> <dst>" 2
    has cp || die "fs_cp_file: missing required command: cp" 2

    command cp -p -- "${src}" "${dst}" 2>/dev/null && return 0
    command cp -p "${src}" "${dst}" 2>/dev/null && return 0

    command cp -- "${src}" "${dst}" 2>/dev/null && return 0
    command cp "${src}" "${dst}" 2>/dev/null && return 0

    die "fs_cp_file: failed: ${src} -> ${dst}" 2

}
fs_cp_dir () {

    local src="${1-}"
    local dst="${2-}"

    [[ -n "${src}" && -n "${dst}" ]] || die "fs_cp_dir: usage: fs_cp_dir <src> <dst>" 2
    has cp || die "fs_cp_dir: missing required command: cp" 2

    command cp -R -p -- "${src}" "${dst}" 2>/dev/null && return 0
    command cp -R -p "${src}" "${dst}" 2>/dev/null && return 0

    command cp -R -- "${src}" "${dst}" 2>/dev/null && return 0
    command cp -R "${src}" "${dst}" 2>/dev/null && return 0

    die "fs_cp_dir: failed: ${src} -> ${dst}" 2

}
fs_rm_file () {

    local f="${1-}"
    [[ -n "${f}" ]] || die "fs_rm_file: missing file" 2
    has rm || die "fs_rm_file: missing required command: rm" 2

    command rm -f -- "${f}" 2>/dev/null && return 0
    command rm -f "${f}" 2>/dev/null && return 0

    die "fs_rm_file: failed: ${f}" 2

}
fs_rm_dir () {

    local d="${1-}"
    [[ -n "${d}" ]] || die "fs_rm_dir: missing dir" 2
    has rm || die "fs_rm_dir: missing required command: rm" 2

    command rm -rf -- "${d}" 2>/dev/null && return 0
    command rm -rf "${d}" 2>/dev/null && return 0

    die "fs_rm_dir: failed: ${d}" 2

}

is_text_file () {

    local file="${1}"

    [[ -f "${file}" ]] || return 1
    [[ -s "${file}" ]] || return 0

    LC_ALL=C grep -Iq . -- "${file}" 2>/dev/null || LC_ALL=C grep -Iq . "${file}" 2>/dev/null

}
dir_exists () {

    local path=""
    path="$(fs_path_expand "${1:-}")"
    [[ -n "${path}" ]] || return 1
    [[ -d "${path}" ]]

}
file_exists () {

    local path=""
    path="$(fs_path_expand "${1:-}")"
    [[ -n "${path}" ]] || return 1
    [[ -f "${path}" ]]

}
new_dir () {

    local d="${1-}"
    [[ -n "${d}" ]] || die "new_dir: missing dir path" 2

    fs_mkdir_p "${d}"

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
remove_file () {

    local f="${1-}"
    [[ -n "${f}" ]] || die "remove_file: missing file" 2

    f="$(fs_path_expand "${f}")"

    fs_guard_rm_target "${f}"

    [[ -e "${f}" ]] || return 0
    [[ -f "${f}" || -L "${f}" ]] || die "remove_file: not a file: ${f}" 2

    fs_rm_file "${f}"

}
move_dir () {

    local src="${1-}"
    local dst="${2-}"

    [[ -n "${src}" && -n "${dst}" ]] || die "move_dir: usage: move_dir <src> <dst>" 2

    src="$(fs_path_expand "${src}")"
    dst="$(fs_path_expand "${dst}")"

    [[ -d "${src}" ]] || die "move_dir: missing source dir: ${src}" 2

    fs_mv "${src}" "${dst}"

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
copy_dir () {

    local src="${1-}"
    local dst="${2-}"

    [[ -n "${src}" && -n "${dst}" ]] || die "copy_dir: usage: copy_dir <src> <dst>" 2

    src="$(fs_path_expand "${src}")"
    dst="$(fs_path_expand "${dst}")"

    [[ -d "${src}" ]] || die "copy_dir: missing source dir: ${src}" 2

    fs_cp_dir "${src}" "${dst}"

}
copy_file () {

    local src="${1-}"
    local dst="${2-}"

    [[ -n "${src}" && -n "${dst}" ]] || die "copy_file: usage: copy_file <src> <dst>" 2

    src="$(fs_path_expand "${src}")"
    dst="$(fs_path_expand "${dst}")"

    [[ -f "${src}" ]] || die "copy_file: missing source file: ${src}" 2

    fs_cp_file "${src}" "${dst}"

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
        [[ -n "${n}" ]] || n="$(command stat -f '%z' -- "${f}" 2>/dev/null || command stat -f '%z' "${f}" 2>/dev/null || true)"
        [[ -n "${n}" ]] && { printf '%s' "${n}"; return 0; }

    fi

    has wc || die "file_size: missing required command: wc" 2

    n="$(command wc -c < "${f}" 2>/dev/null | tr -d '[:space:]' || true)"
    [[ -n "${n}" ]] || die "file_size: failed: ${f}" 2

    printf '%s' "${n}"

}
files_count () {

    local d="${1-}"
    [[ -n "${d}" ]] || die "files_count: missing dir" 2
    [[ -d "${d}" ]] || die "files_count: not a dir: ${d}" 2

    has find || die "files_count: missing required command: find" 2

    local n=0
    while IFS= read -r -d '' _; do
        n=$(( n + 1 ))
    done < <(
        command find "${d}" -type f \
            ! -path '*/.git/*' ! -path '*/.hg/*' ! -path '*/.svn/*' \
            -print0 2>/dev/null || true
    )

    printf '%s' "${n}"

}
trim_file () {

    local f="${1-}"
    [[ -n "${f}" ]] || die "trim_file: missing file" 2
    [[ -f "${f}" ]] || die "trim_file: not a file: ${f}" 2

    has awk || die "trim_file: missing required command: awk" 2
    has mktemp || die "trim_file: missing required command: mktemp" 2

    local dir=""
    dir="$(fs_path_dirname "${f}")"

    local base=""
    base="$(fs_path_basename "${f}")"

    local tmp=""
    tmp="$(mktemp "${dir}/.${base}.trim.XXXXXXXX" 2>/dev/null || true)"
    [[ -n "${tmp}" ]] || tmp="$(mktemp -t "${base}.trim.XXXXXXXX" 2>/dev/null || true)"
    [[ -n "${tmp}" ]] || die "trim_file: mktemp failed" 2

    LC_ALL=C awk '
        { sub(/[[:space:]]+$/, "", $0); lines[NR] = $0 }
        END {
            s = 1
            while (s <= NR && lines[s] == "") s++
            e = NR
            while (e >= s && lines[e] == "") e--
            for (i = s; i <= e; i++) print lines[i]
        }
    ' < "${f}" > "${tmp}" || { fs_rm_file "${tmp}" 2>/dev/null || true; die "trim_file: failed: ${f}" 2; }

    fs_mv "${tmp}" "${f}"

}
replace () {

    ensure_pkg perl grep 2>&1

    local file="${1:-}"
    local old="${2:-}"
    local new="${3-}"

    [[ -n "${file}" ]] || die "replace: missing file" 2
    [[ -f "${file}" ]] || die "replace: file not found: ${file}" 2
    [[ -n "${old}"  ]] || die "replace: missing old_word" 2
    [[ -L "${file}" ]] && return 2
    [[ "${old}" != "${new}" ]] || return 1

    is_text_file "${file}" || return 2
    LC_ALL=C grep -Fq -- "${old}" "${file}" 2>/dev/null || return 1

    perl -i -pe '
        BEGIN {
            $old = $ARGV[0];
            $new = $ARGV[1];
            shift @ARGV; shift @ARGV;

            $new =~ s/\\/\\\\/g;
            $new =~ s/\$/\\\$/g;
            $new =~ s/\@/\\\@/g;
        }
        s/\Q$old\E/$new/g;
    ' "${old}" "${new}" "${file}" || die "replace: failed: ${file}" 2

    log "${file}: (${old}) -> (${new})"
    return 0

}
replace_all () {

    ensure_pkg find perl grep 2>&1

    local ignore_arg=""
    local -a ignore_raw=()

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -i|--ignore)
                ignore_arg="${2-}"
                [[ -n "${ignore_arg}" ]] || die "replace_all: missing value for --ignore" 2
                ignore_raw+=( "${ignore_arg}" )
                shift 2 || true
            ;;
            --)
                shift
                break
            ;;
            -*)
                die "replace_all: unknown option: $1" 2
            ;;
            *)
                break
            ;;
        esac
    done

    local old="${1:-}" new="${2-}" root="${3:-.}"

    [[ -n "${old}" ]] || die "replace_all: missing old_word" 2
    [[ -e "${root}" ]] || die "replace_all: path not found: ${root}" 2
    [[ "${old}" != "${new}" ]] || return 1

    local total=0 changed=0 missed=0 skipped=0 failed=0
    local file="" rc=0

    if [[ -f "${root}" ]]; then

        total=1

        if [[ -L "${root}" ]]; then
            skipped=1
        else
            is_text_file "${root}" || skipped=1
            if (( skipped == 0 )); then
                LC_ALL=C grep -Fq -- "${old}" "${root}" 2>/dev/null || missed=1
                if (( missed == 0 )); then
                    perl -i -pe '
                        BEGIN {
                            $old = $ARGV[0];
                            $new = $ARGV[1];
                            shift @ARGV; shift @ARGV;

                            $new =~ s/\\/\\\\/g;
                            $new =~ s/\$/\\\$/g;
                            $new =~ s/\@/\\\@/g;
                        }
                        s/\Q$old\E/$new/g;
                    ' "${old}" "${new}" "${root}" || failed=1
                    (( failed == 0 )) && changed=1
                fi
            fi
        fi

        log
        log "replace_all: total=${total} changed=${changed} missed=${missed} skipped=${skipped} failed=${failed}"

        (( failed == 0 )) || return 2
        (( changed > 0 )) && return 0 || return 1

    fi

    local root_clean="${root%/}"
    [[ -n "${root_clean}" ]] || root_clean="/"

    local s="" part="" trimmed=""
    local -a parts=()
    local -a ignore_list=(".git" "target" "node_modules" "dist" "build" ".next" ".venv" "venv" ".vscode" "__pycache__")

    for s in "${ignore_raw[@]-}"; do

        IFS=',' read -r -a parts <<< "${s}"

        for part in "${parts[@]-}"; do

            trimmed="${part#"${part%%[![:space:]]*}"}"
            trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
            [[ -n "${trimmed}" ]] || continue

            ignore_list+=( "${trimmed}" )

        done

    done

    local -a dtests=()
    local -a fexcl=()
    local item="" p=""

    for item in "${ignore_list[@]-}"; do

        if [[ "${item}" == /* ]]; then

            p="${root_clean}${item}"
            dtests+=( -path "${p}" -o )
            fexcl+=( ! -path "${p}" )

        elif [[ "${item}" == */* ]]; then

            dtests+=( -path "*/${item}" -o )
            fexcl+=( ! -path "*/${item}" )

        else

            dtests+=( -name "${item}" -o )
            fexcl+=( ! -name "${item}" )

        fi

    done

    local -a find_cmd=( find -H "${root_clean}" )

    if (( ${#dtests[@]} )); then
        unset "dtests[${#dtests[@]}-1]"
        find_cmd+=( -type d "(" "${dtests[@]}" ")" -prune -o )
    fi

    find_cmd+=( -type f ! -lname '*' )

    if (( ${#fexcl[@]} )); then
        find_cmd+=( "${fexcl[@]}" )
    fi

    find_cmd+=( -print0 )

    while IFS= read -r -d '' file; do

        [[ -n "${file}" ]] || continue
        total=$(( total + 1 ))

        if [[ -L "${file}" ]]; then
            skipped=$(( skipped + 1 ))
            continue
        fi

        is_text_file "${file}" || { skipped=$(( skipped + 1 )); continue; }

        LC_ALL=C grep -Fq -- "${old}" "${file}" 2>/dev/null || { missed=$(( missed + 1 )); continue; }

        perl -i -pe '
            BEGIN {
                $old = $ARGV[0];
                $new = $ARGV[1];
                shift @ARGV; shift @ARGV;

                $new =~ s/\\/\\\\/g;
                $new =~ s/\$/\\\$/g;
                $new =~ s/\@/\\\@/g;
            }
            s/\Q$old\E/$new/g;
        ' "${old}" "${new}" "${file}" || { failed=$(( failed + 1 )); continue; }

        changed=$(( changed + 1 ))

    done < <( command "${find_cmd[@]}" 2>/dev/null || true )

    log
    log "replace_all: total=${total} changed=${changed} missed=${missed} skipped=${skipped} failed=${failed}"

    (( failed == 0 )) || return 2
    (( changed > 0 )) && return 0 || return 1

}
replace_map () {

    ensure_pkg perl grep 2>&1

    local file="${1:-}"
    local map_name="${2-}"

    [[ -n "${file}" ]] || die "replace_map: missing file" 2
    [[ -f "${file}" ]] || die "replace_map: file not found: ${file}" 2
    [[ -n "${map_name}" ]] || die "replace_map: missing map name" 2
    [[ -L "${file}" ]] && return 2

    is_text_file "${file}" || return 2

    local -n m="${map_name}"
    local -a pairs=()
    local -a grep_args=()

    local k=""
    for k in "${!m[@]}"; do

        [[ -n "${k}" ]] || continue
        [[ -n "${m[${k}]}" ]] || continue

        pairs+=( "${k}" "${m[${k}]}" )
        grep_args+=( -e "${k}" )

    done

    (( ${#pairs[@]} )) || return 1
    LC_ALL=C grep -Fq "${grep_args[@]}" -- "${file}" 2>/dev/null || return 1

    perl -i -pe '
        BEGIN {
            my $i = 0;
            for ( $i = 0; $i < @ARGV; $i++ ) { last if $ARGV[$i] eq "--"; }
            die "replace_map: missing -- delimiter\n" if $i == @ARGV;

            my @pairs = @ARGV[0 .. $i - 1];
            @ARGV = @ARGV[$i + 1 .. $#ARGV];

            die "replace_map: pairs mismatch\n" if @pairs % 2;

            our %map = ();
            for ( my $j = 0; $j < @pairs; $j += 2 ) {
                my $old = $pairs[$j];
                my $new = $pairs[$j + 1];

                $new =~ s/\\/\\\\/g;
                $new =~ s/\$/\\\$/g;
                $new =~ s/\@/\\\@/g;

                $map{$old} = $new;
            }

            my @keys = sort { length($b) <=> length($a) } keys %map;
            our $re = join("|", map { quotemeta($_) } @keys);
            our $changed = 0;
        }

        if ( $re ne "" ) {
            $changed += s/($re)/$map{$1}/g;
        }

        END { exit($changed == 0 ? 1 : 0); }
    ' "${pairs[@]}" -- "${file}" || {
        local rc=$?
        (( rc == 1 )) && return 1
        die "replace_map: failed to replace map in ${file}" 2
    }

    success "Ok: map replaced in ${file}"
    return 0

}
replace_all_map () {

    ensure_pkg find perl grep 2>&1

    local ignore_arg=""
    local -a ignore_raw=()

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -i|--ignore)
                ignore_arg="${2-}"
                [[ -n "${ignore_arg}" ]] || die "replace_all_map: missing value for --ignore" 2
                ignore_raw+=( "${ignore_arg}" )
                shift 2 || true
            ;;
            --)
                shift
                break
            ;;
            -*)
                die "replace_all_map: unknown option: $1" 2
            ;;
            *)
                break
            ;;
        esac
    done

    local root="${1:-.}"
    local map_name="${2-}"

    [[ -n "${map_name}" ]] || die "replace_all_map: missing map name" 2
    [[ -e "${root}" ]] || die "replace_all_map: path not found: ${root}" 2

    local total=0 changed=0 missed=0 skipped=0 failed=0
    local file="" rc=0

    if [[ -f "${root}" ]]; then

        total=1

        replace_map "${root}" "${map_name}"
        rc=$?

        case "${rc}" in
            0) changed=1 ;;
            1) missed=1 ;;
            2) skipped=1 ;;
            *) failed=1 ;;
        esac

        success "Ok: total=${total} changed=${changed} missed=${missed} skipped=${skipped} failed=${failed}"

        (( failed == 0 )) || return 2
        (( changed > 0 )) && return 0 || return 1

    fi

    local root_clean="${root%/}"
    [[ -n "${root_clean}" ]] || root_clean="/"

    local -n m="${map_name}"

    local -a pairs=()
    local -a grep_args=()

    local k=""
    for k in "${!m[@]}"; do

        [[ -n "${k}" ]] || continue
        [[ -n "${m[${k}]}" ]] || continue

        pairs+=( "${k}" "${m[${k}]}" )
        grep_args+=( -e "${k}" )

    done

    (( ${#pairs[@]} )) || return 1

    local s="" part="" trimmed=""
    local -a parts=()
    local -a ignore_list=(".git" "target" "node_modules" "dist" "build" ".next" ".venv" "venv" ".vscode" "__pycache__")

    for s in "${ignore_raw[@]-}"; do

        IFS=',' read -r -a parts <<< "${s}"

        for part in "${parts[@]-}"; do

            trimmed="${part#"${part%%[![:space:]]*}"}"
            trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
            [[ -n "${trimmed}" ]] || continue

            ignore_list+=( "${trimmed}" )

        done

    done

    local -a dtests=()
    local -a fexcl=()
    local item="" p=""

    for item in "${ignore_list[@]-}"; do

        if [[ "${item}" == /* ]]; then

            p="${root_clean}${item}"
            dtests+=( -path "${p}" -o )
            fexcl+=( ! -path "${p}" )

        elif [[ "${item}" == */* ]]; then

            dtests+=( -path "*/${item}" -o )
            fexcl+=( ! -path "*/${item}" )

        else

            dtests+=( -name "${item}" -o )
            fexcl+=( ! -name "${item}" )

        fi

    done

    local -a find_cmd=( find -H "${root_clean}" )

    if (( ${#dtests[@]} )); then
        unset "dtests[${#dtests[@]}-1]"
        find_cmd+=( -type d "(" "${dtests[@]}" ")" -prune -o )
    fi

    find_cmd+=( -type f ! -lname '*' )

    if (( ${#fexcl[@]} )); then
        find_cmd+=( "${fexcl[@]}" )
    fi

    find_cmd+=( -print0 )

    while IFS= read -r -d '' file; do

        [[ -n "${file}" ]] || continue
        total=$(( total + 1 ))

        if [[ -L "${file}" ]]; then
            skipped=$(( skipped + 1 ))
            continue
        fi

        is_text_file "${file}" || { skipped=$(( skipped + 1 )); continue; }

        LC_ALL=C grep -Fq "${grep_args[@]}" -- "${file}" 2>/dev/null || { missed=$(( missed + 1 )); continue; }

        perl -i -pe '
            BEGIN {
                my $i = 0;
                for ( $i = 0; $i < @ARGV; $i++ ) { last if $ARGV[$i] eq "--"; }
                die "replace_all_map: missing -- delimiter\n" if $i == @ARGV;

                my @pairs = @ARGV[0 .. $i - 1];
                @ARGV = @ARGV[$i + 1 .. $#ARGV];

                die "replace_all_map: pairs mismatch\n" if @pairs % 2;

                our %map = ();
                for ( my $j = 0; $j < @pairs; $j += 2 ) {
                    my $old = $pairs[$j];
                    my $new = $pairs[$j + 1];

                    $new =~ s/\\/\\\\/g;
                    $new =~ s/\$/\\\$/g;
                    $new =~ s/\@/\\\@/g;

                    $map{$old} = $new;
                }

                my @keys = sort { length($b) <=> length($a) } keys %map;
                our $re = join("|", map { quotemeta($_) } @keys);
                our $changed = 0;
            }

            if ( $re ne "" ) {
                $changed += s/($re)/$map{$1}/g;
            }

            END { exit($changed == 0 ? 1 : 0); }
        ' "${pairs[@]}" -- "${file}" || {
            rc=$?
            (( rc == 1 )) && { missed=$(( missed + 1 )); continue; }
            failed=$(( failed + 1 ))
            continue
        }

        changed=$(( changed + 1 ))

    done < <( command "${find_cmd[@]}" 2>/dev/null || true )

    success "Ok: total=${total} changed=${changed} missed=${missed} skipped=${skipped} failed=${failed}"

    (( failed == 0 )) || return 2
    (( changed > 0 )) && return 0 || return 1

}
