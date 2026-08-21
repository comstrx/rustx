#!/usr/bin/env bash

[[ "${BASH_SOURCE[0]}" != "${0}" ]] || { printf '%s\n' "installer.sh: this file should not be run externally." >&2; exit 2; }
[[ -n "${INSTALLER_LOADED:-}" ]] && return 0
INSTALLER_LOADED=1

__dir__="${BASH_SOURCE[0]%/*}"
[[ "${__dir__}" == "${BASH_SOURCE[0]}" ]] && __dir__="."
__dir__="$(cd -- "${__dir__}" && pwd -P)"
source "${__dir__}/boot.sh"

ensure_line_once () {

    ensure_pkg grep 2>&1

    local file="${1:-}"
    local line="${2:-}"

    [[ -n "${file}" ]] || die "ensure_line_once: missing file" 2
    [[ -n "${line}" ]] || die "ensure_line_once: missing line" 2

    [[ -L "${file}" ]] && die "Refusing to modify symlink: ${file}" 2

    ensure_file "${file}"

    LC_ALL=C grep -Fqx -- "${line}" "${file}" 2>/dev/null && return 0
    LC_ALL=C grep -Fqx -- "${line}"$'\r' "${file}" 2>/dev/null && return 0

    printf '%s\n' "${line}" >> "${file}" || die "Failed writing: ${file}" 2

}
ensure_path_once () {

    local rc="${1:-}"
    local alias_name="${2:-}"

    [[ -n "${rc}" ]] || die "ensure_path_once: missing rc" 2
    [[ -L "${rc}" ]] && die "Refusing to modify symlink: ${rc}" 2

    ensure_file "${rc}"
    ensure_line_once "${rc}" "# ${alias_name}"

    case "${rc}" in
        */.config/fish/config.fish) ensure_line_once "${rc}" 'set -gx PATH $HOME/.local/bin $PATH' ;;
        *) ensure_line_once "${rc}" 'export PATH="$HOME/.local/bin:$PATH"' ;;
    esac

}
ensure_extracted () {

    local template_name="template" here="$(pwd -P)" parent="" parent_template=""
    parent="$(cd .. && pwd -P 2>/dev/null)" || parent=""

    if [[ -n "${parent}" && -d "${parent}/${template_name}" ]]; then
        parent_template="$(cd "${parent}/${template_name}" && pwd -P 2>/dev/null)" || parent_template=""
    fi

    local root="${here}"
    [[ -n "${parent_template}" && "${parent_template}" == "${here}" ]] && root="${parent}"

    if [[ -f "${root}/scripts/install.sh" ]]; then

        [[ -d "${root}/${template_name}" ]] && rm -rf -- "${root}/${template_name}" || true
        return 0

    fi

    [[ -f "${root}/${template_name}/scripts/install.sh" ]] || return 0

    [[ "${root}" != "/" && "${root}" != "${HOME-}" ]] || die "Refusing to wipe unsafe dir: ${root}"

    find "${root}" -mindepth 1 -maxdepth 1 ! -name "${template_name}" -exec rm -rf -- {} + || die "Failed to clean root: ${root}"
   
    ( cd "${root}/${template_name}" && tar -cf - . ) | ( cd "${root}" && tar -xpf - ) || die "Failed to promote template into root: ${root}"

    cd -- "${root}" || die "Failed to cd to root: ${root}" 2
    rm -rf -- "${root}/${template_name}" || die "Failed to remove template/: ${root}/${template_name}"

    exec bash "${root}/scripts/install.sh" "$@"

}
install_launcher () {

    ensure_pkg chmod mkdir 2>&1

    local root="${1:-}"
    local alias_name="${2:-}"

    [[ -n "${root}" ]] || die "install_launcher: missing root" 2
    is_valid_alias "${alias_name}" || die "Invalid alias: ${alias_name}" 2

    local run_sh="${root}/scripts/run.sh"
    [[ -f "${run_sh}" ]] || die "Missing: ${run_sh}" 2

    chmod +x "${run_sh}" >/dev/null 2>&1 || true

    local bin_dir="$(home_path)/.local/bin"
    local bin="${bin_dir}/${alias_name}"
    ensure_dir "${bin_dir}"

    if [[ -e "${bin}" && ! -f "${bin}" ]]; then
        die "Refusing: target exists but not a file: ${bin}" 2
    fi
    if [[ -L "${bin}" ]]; then
        die "Refusing to overwrite symlink: ${bin}" 2
    fi
    if [[ -e "${bin}" ]] && ! (( YES_ENV )); then
        confirm "Overwrite ${bin}?" || die "Canceled." 2
    fi

    local root_q=""
    printf -v root_q '%q' "${root}"

    printf '%s\n' \
        '#!/usr/bin/env bash' \
        'set -Eeuo pipefail' \
        '' \
        "ROOT=${root_q}" \
        '' \
        'exec /usr/bin/env bash "${ROOT}/scripts/run.sh" "$@"' \
        > "${bin}" || die "Failed writing: ${bin}" 2

    chmod +x "${bin}" || die "chmod failed: ${bin}" 2
    printf '%s\n' "${bin}"

}
default_branch () {

    local root="${1:-}"
    local b=""

    if has git && [[ -e "${root}/.git" ]]; then

        b="$(cd -- "${root}" && git symbolic-ref -q --short refs/remotes/origin/HEAD 2>/dev/null || true)"
        b="${b#origin/}"

        [[ -n "${b}" ]] || b="$(cd -- "${root}" && git rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
        [[ "${b}" == "HEAD" ]] && b=""

    fi

    [[ -n "${b}" ]] || b="main"
    printf '%s' "${b}"

}
gh_blob_url () {

    local repo_url="${1:-}"
    local branch="${2:-}"
    local rel="${3:-}"

    rel="${rel#/}"
    printf '%s/blob/%s/%s' "${repo_url}" "${branch}" "${rel}"

}
gh_tree_url () {

    local repo_url="${1:-}"
    local branch="${2:-}"
    local rel="${3:-}"

    rel="${rel#/}"
    printf '%s/tree/%s/%s' "${repo_url}" "${branch}" "${rel}"

}
placeholders () {

    local root="${1:-}"
    local alias="${2:-}"
    local user="${3-}"
    local name="${4-}"
    local repo="${5-}"
    local branch="${6-}"
    local description="${7-}"
    local github_base="${8:-}"
    local discord_url="${9-}"
    local docs_url="${10-}"
    local site_url="${11-}"

    local -A ph_map=()

    append () {

        local k="${1-}"
        local v="${2-}"

        [[ -n "${k}" ]] || return 0
        [[ -n "${v}" ]] || return 0

        k="${k,,}"

        ph_map["__${k}__"]="${v}"
        ph_map["__${k^^}__"]="${v}"
        ph_map["{{${k}}}"]="${v}"
        ph_map["{{${k^^}}}"]="${v}"

    }

    append "alias" "${alias}"
    append "branch" "${branch}"
    append "user" "${user}"
    append "repo" "${repo}"
    append "name" "${name}"
    append "description" "${description}"
    append "docs_url" "${docs_url}"
    append "site_url" "${site_url}"
    append "discord_url" "${discord_url}"

    if [[ -n "${user}" && -n "${repo}" ]]; then

        local security_url="" support_url="" readme_url="" contributing_url=""
        local coc_url="" changelog_url="" pr_tmpl_url="" issue_templates_url=""

        local repo_url="${github_base}/${user}/${repo}"
        local issues_url="${repo_url}/issues"
        local new_issue_url="${repo_url}/issues/new/choose"
        local discussions_url="${repo_url}/discussions"
        local community_url="${repo_url}/graphs/community"

        local categories_url="${repo_url}/discussions/categories"
        local announcements_url="${repo_url}/discussions/categories/announcements"
        local general_url="${repo_url}/discussions/categories/general"
        local ideas_url="${repo_url}/discussions/categories/ideas"
        local polls_url="${repo_url}/discussions/categories/polls"
        local qa_url="${repo_url}/discussions/categories/q-a"
        local show_and_tell_url="${repo_url}/discussions/categories/show-and-tell"

        append "repo_url" "${repo_url}"
        append "issues_url" "${issues_url}"
        append "new_issue_url" "${new_issue_url}"
        append "discussions_url" "${discussions_url}"
        append "community_url" "${community_url}"
        append "categories_url" "${categories_url}"
        append "announcements_url" "${announcements_url}"
        append "general_url" "${general_url}"
        append "ideas_url" "${ideas_url}"
        append "polls_url" "${polls_url}"
        append "qa_url" "${qa_url}"
        append "show_and_tell_url" "${show_and_tell_url}"
        append "bug_report_url" "${new_issue_url}"
        append "feature_request_url" "${new_issue_url}"

        if [[ -f "${root}/SECURITY.md" ]]; then
            append "security_url" "$(gh_blob_url "${repo_url}" "${branch}" "SECURITY.md")"
        else
            append "security_url" "${repo_url}/security"
        fi

        if [[ -f "${root}/.github/SUPPORT.md" ]]; then
            append "support_url" "$(gh_blob_url "${repo_url}" "${branch}" ".github/SUPPORT.md")"
        elif [[ -f "${root}/SUPPORT.md" ]]; then
            append "support_url" "$(gh_blob_url "${repo_url}" "${branch}" "SUPPORT.md")"
        else
            append "support_url" "${discussions_url}"
        fi

        if [[ -f "${root}/CONTRIBUTING.md" ]]; then
            append "contributing_url" "$(gh_blob_url "${repo_url}" "${branch}" "CONTRIBUTING.md")"
        elif [[ -f "${root}/.github/CONTRIBUTING.md" ]]; then
            append "contributing_url" "$(gh_blob_url "${repo_url}" "${branch}" ".github/CONTRIBUTING.md")"
        fi

        if [[ -f "${root}/CODE_OF_CONDUCT.md" ]]; then
            append "code_of_conduct_url" "$(gh_blob_url "${repo_url}" "${branch}" "CODE_OF_CONDUCT.md")"
        elif [[ -f "${root}/.github/CODE_OF_CONDUCT.md" ]]; then
            append "code_of_conduct_url" "$(gh_blob_url "${repo_url}" "${branch}" ".github/CODE_OF_CONDUCT.md")"
        fi

        if [[ -f "${root}/README.md" ]]; then
            append "readme_url" "$(gh_blob_url "${repo_url}" "${branch}" "README.md")"
        fi
        if [[ -f "${root}/CHANGELOG.md" ]]; then
            append "changelog_url" "$(gh_blob_url "${repo_url}" "${branch}" "CHANGELOG.md")"
        fi
        if [[ -f "${root}/.github/PULL_REQUEST_TEMPLATE.md" ]]; then
            append "pull_request_template_url" "$(gh_blob_url "${repo_url}" "${branch}" ".github/PULL_REQUEST_TEMPLATE.md")"
        fi
        if [[ -d "${root}/.github/ISSUE_TEMPLATE" ]]; then
            append "issue_templates_url" "$(gh_tree_url "${repo_url}" "${branch}" ".github/ISSUE_TEMPLATE")"
        fi

    fi

    replace_all_map "${root}" ph_map

}
usage () {

    printf '%s\n' \
        "" \
        "Usage:" \
        "" \
        "  install.sh            [--yes|-y] [--alias|-a NAME] [--user USER] [--repo REPO] [--name NAME] [--branch BRANCH]" \
        "" \
        "                        [--github-base URL] [--discord URL] [--docs URL] [--site URL]" \
        "" \
        "Options:" \
        "" \
        "      --alias            Command name" \
        "      --name             Project/workspace name (placeholders)" \
        "      --description      Project/workspace description (placeholders)" \
        "      --user             GitHub user/org (placeholders + URLs)" \
        "      --repo             GitHub repo name (placeholders + URLs)" \
        "      --branch           GitHub Default branch (used for blob/tree URLs)" \
        "      --github-base      GitHub base (default: https://github.com)" \
        "      --discord          Discord invite/server URL" \
        "      --docs             Docs URL" \
        "      --site             Project website URL" \
        "  -y, --yes              Non-interactive (assume yes)" \
        "  -h, --help             Show help" \
        ""

}
install () {

    cd_root
    ensure_extracted "$@"

    source <(parse "$@" -- alias=rustx name=rustx user=comstrx repo=rustx branch=main description discord docs site github_base )

    local root="${ROOT_DIR}"
    [[ -n "${root}" ]] || die "ROOT_DIR is empty" 2

    github_base="${github_base%/}"
    [[ -n "${github_base}" ]] || github_base="https://github.com"
    [[ -n "${branch}" ]] || branch="$(default_branch "${root}")"

    [[ -n "${user}" ]] && user="$(slugify "${user}")"
    [[ -n "${repo}" ]] && repo="$(slugify "${repo}")"
    [[ -n "${name}" ]] && name="$(slugify "${name}")"

    [[ -z "${repo}" && -n "${name}" ]] && repo="${name}"
    [[ -z "${name}" && -n "${repo}" ]] && name="${repo}"

    [[ -n "${description}" ]] || description="The foundational Rust core for the ToolX ecosystem."
    [[ -n "${site}" ]] || site="${github_base}/${user}/${repo}"
    [[ -n "${docs}" ]] || docs="${site}/tree/${branch}/docs"

    is_valid_alias "${alias}" || die "Invalid alias: ${alias}" 2

    local bin_path="$(install_launcher "${root}" "${alias}")"
    local rc="$(rc_path)"

    ensure_path_once "${rc}" "${alias}"

    placeholders \
        "${root}" \
        "${alias}" \
        "${user}" \
        "${name}" \
        "${repo}" \
        "${branch}" \
        "${description}" \
        "${github_base}" \
        "${discord}" \
        "${docs}" \
        "${site}"

    printf '%s\n' \
        "OK: installed ${bin_path}" \
        "OK: updated ${rc}" \
        "Reload: source \"${rc}\"" \
        "Then: ${alias} --help"

}
