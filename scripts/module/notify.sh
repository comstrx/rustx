#!/usr/bin/env bash

cmd_notify_help () {

    info_ln "Notify :\n"

    printf '    %s\n' \
        "notify                     * Send notification" \
        "    --platform               Platform <telegram|slack|discord|webhook>, Default: every platform with credentials configured" \
        "    --token                  Token value (telegram mode)" \
        "    --chat                   Chat id (telegram mode)" \
        "    --webhook                Webhook url (slack|discord|webhook url)" \
        "    --status                 Workflow status" \
        "    --title                  CI workflow name" \
        "    --retries                Default: 3" \
        "    --delay                  Default: 1" \
        ''

}

notify_message () {

    local status="${1:-}" title="${2:-}"

    local ref="${REF:-${GITHUB_REF:-}}"
    local sha="${SHA:-${GITHUB_SHA:-}}"
    local url="${URL:-${GITHUB_URL:-${GITHUB_WORKFLOW_URL:-${WORKFLOW_URL:-}}}}"
    local started_at="${STARTED_AT:-${RUN_STARTED_AT:-${GITHUB_RUN_STARTED_AT:-}}}"
    local finished_at="${FINISHED_AT:-${RUN_FINISHED_AT:-${GITHUB_RUN_FINISHED_AT:-}}}"
    local run_id="${RUN_ID:-${GITHUB_RUN_ID:-}}"
    local server_url="${SERVER_URL:-${GITHUB_SERVER_URL:-}}"
    local repo="${REPOSITORY:-${GITHUB_REPOSITORY:-}}"
    local repo_name="${repo##*/}"
    local status_icon="🤔" status_label="Unknown"
    local duration="0s" date_str=""

    date_str="$(date -u +%F 2>/dev/null || date +%F)"

    [[ -n "${status}" ]] || status="${STATUS:-${GITHUB_STATUS:-}}"
    [[ -n "${title}" ]] || title="${WORKFLOW_NAME:-${GITHUB_WORKFLOW_NAME:-CI}} Workflow"
    [[ -z "${url}" && -n "${server_url}" && -n "${repo}" && -n "${run_id}" ]] && url="${server_url}/${repo}/actions/runs/${run_id}"

    case "${status,,}" in
        success|succeeded|ok|passed|pass)    status_icon="✅" ; status_label="Success" ;;
        warn|warning|warnings)     status_icon="⚠️" ; status_label="Warning" ;;
        fail|failed|failure|error)  status_icon="❌" ; status_label="Failed" ;;
        cancel|canceled|cancelled) status_icon="🟡" ; status_label="Cancelled" ;;
        skip|skipped)              status_icon="⚪" ; status_label="Skipped" ;;
    esac
    case "${ref}" in
        refs/heads/*) ref="${ref#refs/heads/}" ;;
        refs/tags/*)  ref="${ref#refs/tags/}"  ;;
    esac
    if [[ -n "${started_at}" ]]; then

        local st="${started_at}" ft="${finished_at}"

        [[ "${st}" == *.*Z ]] && st="${st%%.*}Z"
        [[ "${ft}" == *.*Z ]] && ft="${ft%%.*}Z"

        local start_s="" end_s=""

        start_s="$(date -u -d "${st}" +%s 2>/dev/null || date -u -j -f "%Y-%m-%dT%H:%M:%SZ" "${st}" +%s 2>/dev/null || true)"
        end_s="$(date -u -d "${ft}" +%s 2>/dev/null || date -u -j -f "%Y-%m-%dT%H:%M:%SZ" "${ft}" +%s 2>/dev/null || true)"

        [[ -n "${end_s}" ]] || end_s="$(date -u +%s 2>/dev/null || date +%s)"

        if [[ -n "${start_s}" ]]; then

            local delta=$(( end_s - start_s ))
            (( delta < 0 )) && delta=0

            local h=$(( delta / 3600 ))
            local m=$(( (delta % 3600) / 60 ))
            local s=$(( delta % 60 ))

            if (( h > 0 )); then duration="${h}h ${m}m ${s}s"
            elif (( m > 0 )); then duration="${m}m ${s}s"
            else duration="${s}s"
            fi

        fi

    fi

    [[ -n "${url}" ]] || url="--"
    [[ -n "${ref}" ]] || ref="--"
    [[ -n "${repo}" ]] || repo="--"
    [[ -n "${repo_name}" ]] || repo_name="--"
    [[ -n "${sha}" ]] && sha="${sha:0:7}" || sha="--"

    printf '%s\n' \
        "==>" \
        "" \
        "💥 ${title} :" \
        "" \
        "      ( Status )      :  ${status_icon} ${status_label}" \
        "" \
        "      ( Duration )  :  ${duration}" \
        "" \
        "      ( Date )         :  ${date_str}" \
        "" \
        "      ( Repo )        :  ${repo}" \
        "" \
        "      ( Commit )   :  ${repo_name}@${ref} • ${sha}" \
        "" \
        "      ( URL )          :  ${url}" \
        "" \
        "==>"

}
notify_post () {

    local -n curl_args="${1}"
    local label="${2:-}" url="${3:-}"

    shift 3 || true

    [[ -n "${url}" ]] || die "notify_post: missing url" 2

    case "${url}" in
        *[[:space:]]*|*'"'*) die "notify: ${label}: url contains whitespace or a quote" 2 ;;
    esac

    local out="" rc=0

    out="$(printf 'url = "%s"\n' "${url}" | curl "${curl_args[@]}" -K - -X POST "$@" 2>&1)" || rc=$?

    if (( rc )); then
        error "notify: ${label} failed ( curl exit ${rc} ) : ${out}"
        return "${rc}"
    fi

    return 0

}
notify_json_hook () {

    ensure curl jq

    local args_ref="${1}" label="${2:-}" webhook="${3:-}" key="${4:-}" msg="${5:-}"

    [[ -n "${webhook}" ]] || die "notify_${label}: missing ${label} webhook"

    local body=""
    body="$(jq -cn --arg k "${key}" --arg t "${msg}" '{($k):$t}')"

    notify_post "${args_ref}" "${label}" "${webhook}" -H "Content-Type: application/json" --data "${body}"

}

notify_telegram () {

    ensure curl

    local args_ref="${1}"
    local token="${2:-}" chat="${3:-}" msg="${4:-}"

    [[ -n "${token}" ]] || token="${TELEGRAM_TOKEN:-${TOKEN:-}}"
    [[ -n "${chat}"  ]] || chat="${TELEGRAM_CHAT_ID:-${TELEGRAM_CHAT:-${CHAT_ID:-${CHAT:-}}}}"
    [[ -n "${token}" ]] || die "notify: missing telegram token"
    [[ -n "${chat}"  ]] || die "notify: missing telegram chat"

    notify_post "${args_ref}" telegram "https://api.telegram.org/bot${token}/sendMessage" \
        -d "chat_id=${chat}" --data-urlencode "text=${msg}" -d "disable_web_page_preview=true"

}
notify_slack () {

    local webhook="${2:-}"

    [[ -n "${webhook}" ]] || webhook="${SLACK_WEBHOOK_URL:-${SLACK_WEBHOOK:-${SLACK_URL:-}}}"
    notify_json_hook "${1}" slack "${webhook}" text "${3:-}"

}
notify_discord () {

    local webhook="${2:-}"

    [[ -n "${webhook}" ]] || webhook="${DISCORD_WEBHOOK_URL:-${DISCORD_WEBHOOK:-${DISCORD_URL:-}}}"
    notify_json_hook "${1}" discord "${webhook}" content "${3:-}"

}
notify_webhook () {

    local webhook="${2:-}"

    [[ -n "${webhook}" ]] || webhook="${WEBHOOK_URL:-${WEBHOOK:-}}"
    notify_json_hook "${1}" webhook "${webhook}" text "${3:-}"

}

cmd_notify () {

    source <(parse "$@" -- \
        status title message \
        token chat telegram_token telegram_chat \
        slack_webhook discord_webhook webhook_url webhook \
        retries:int=3 delay:int=1 timeout:float=10 max_time:float=20 retry_max_time:float=60 \
        'platform|platforms:list' \
    )

    local -a args=() plats=() failed=()
    local  msg="" p=""

    telegram_token="${telegram_token:-${token:-${TELEGRAM_TOKEN:-${TOKEN:-}}}}"
    telegram_chat="${telegram_chat:-${chat:-${TELEGRAM_CHAT_ID:-${TELEGRAM_CHAT:-${CHAT_ID:-${CHAT:-}}}}}}"
    slack_webhook="${slack_webhook:-${SLACK_WEBHOOK_URL:-${SLACK_WEBHOOK:-${SLACK_URL:-}}}}"
    discord_webhook="${discord_webhook:-${DISCORD_WEBHOOK_URL:-${DISCORD_WEBHOOK:-${DISCORD_URL:-}}}}"
    webhook_url="${webhook_url:-${WEBHOOK_URL:-${WEBHOOK:-}}}"

    args=(
        -fsS --connect-timeout "${timeout}" --max-time "${max_time}" --retry-max-time "${retry_max_time}"
        --retry "${retries}" --retry-delay "${delay}" --retry-connrefused
    )

    if (( ${#platform[@]} )); then
        plats+=( "${platform[@]}" )
    else
        [[ -n "${telegram_token}" && -n "${telegram_chat}" ]] && plats+=( telegram )
        [[ -n "${slack_webhook}"   ]] && plats+=( slack )
        [[ -n "${discord_webhook}" ]] && plats+=( discord )
        [[ -n "${webhook_url}"     ]] && plats+=( webhook )
    fi

    (( ${#plats[@]} )) || die "Failed to detect platform"

    msg="${message:-"$(notify_message "${status}" "${title}")"}"

    for p in "${plats[@]}"; do

        case "${p,,}" in
            telegram) notify_telegram args "${telegram_token}" "${telegram_chat}" "${msg}" || failed+=( telegram ) ;;
            slack)    notify_slack    args "${slack_webhook:-${webhook}}"         "${msg}" || failed+=( slack ) ;;
            discord)  notify_discord  args "${discord_webhook:-${webhook}}"       "${msg}" || failed+=( discord ) ;;
            webhook)  notify_webhook  args "${webhook_url:-${webhook}}"           "${msg}" || failed+=( webhook ) ;;
            *)        failed+=( "${p}" ) ;;
        esac

    done

    if (( ${#failed[@]} )); then die "Failed to send ( ${failed[*]} ) notification"
    else success "Ok: Notification sent successfully ( ${plats[*]} )"
    fi

}
