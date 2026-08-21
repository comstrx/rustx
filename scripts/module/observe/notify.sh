#!/usr/bin/env bash

notify_help () {

    info_ln "Notify :\n"

    printf '    %s\n' \
        "notify                     * Send notification" \
        ''

}
cmd_notify_help () {

    info_ln "Notify :\n"

    printf '    %s\n' \
        "notify                     * Send notification" \
        "    --platform               Platform <telegram|slack|discord|webhook>, Default: detect env then telegram" \
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
    local duration="0s" date_str="$(date -u +%F 2>/dev/null || date +%F)"

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

        local start_s="$(date -u -d "${st}" +%s 2>/dev/null || date -u -j -f "%Y-%m-%dT%H:%M:%SZ" "${st}" +%s 2>/dev/null || true)"
        local end_s="$(date -u -d "${ft}" +%s 2>/dev/null || date -u -j -f "%Y-%m-%dT%H:%M:%SZ" "${ft}" +%s 2>/dev/null || true)"

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
notify_telegram () {

    ensure curl

    local -n curl_args="${1}"
    local token="${2:-}" chat="${3:-}" msg="${4:-}"

    [[ -n "${token}" ]] || token="${TELEGRAM_TOKEN:-${TOKEN:-}}"
    [[ -n "${chat}"  ]] || chat="${TELEGRAM_CHAT_ID:-${TELEGRAM_CHAT:-${CHAT_ID:-${CHAT:-}}}}"
    [[ -n "${token}" ]] || die "notify: missing telegram token"
    [[ -n "${chat}"  ]] || die "notify: missing telegram chat"

    local -a payload=( -d "chat_id=${chat}" --data-urlencode "text=${msg}" -d "disable_web_page_preview=true" )
    curl "${curl_args[@]}" -X POST "https://api.telegram.org/bot${token}/sendMessage" "${payload[@]}" >/dev/null 2>&1 || return 1

}
notify_slack () {

    ensure curl jq

    local -n curl_args="${1}"
    local webhook="${2:-}" msg="${3:-}"

    [[ -n "${webhook}" ]] || webhook="${SLACK_WEBHOOK_URL:-${SLACK_WEBHOOK:-${SLACK_URL:-}}}"
    [[ -n "${webhook}" ]] || die "notify_slack: missing slack webhook"

    local -a payload=( --data "$(jq -cn --arg t "${msg}" '{text:$t}')" "${webhook}" )
    curl "${curl_args[@]}" -X POST -H "Content-Type: application/json" "${payload[@]}" >/dev/null 2>&1 || return 1

}
notify_discord () {

    ensure curl jq

    local -n curl_args="${1}"
    local webhook="${2:-}" msg="${3:-}"

    [[ -n "${webhook}" ]] || webhook="${DISCORD_WEBHOOK_URL:-${DISCORD_WEBHOOK:-${DISCORD_URL:-}}}"
    [[ -n "${webhook}" ]] || die "notify_discord: missing discord webhook"

    local -a payload=( --data "$(jq -cn --arg t "${msg}" '{content:$t}')" "${webhook}" )
    curl "${curl_args[@]}" -X POST -H "Content-Type: application/json" "${payload[@]}" >/dev/null 2>&1 || return 1

}
notify_webhook () {

    ensure curl jq

    local -n curl_args="${1}"
    local webhook="${2:-}" msg="${3:-}"

    [[ -n "${webhook}" ]] || webhook="${WEBHOOK_URL:-${WEBHOOK:-}}"
    [[ -n "${webhook}" ]] || die "notify_webhook: missing webhook url"

    local -a payload=( --data "$(jq -cn --arg t "${msg}" '{text:$t}')" "${webhook}" )
    curl "${curl_args[@]}" -X POST -H "Content-Type: application/json" "${payload[@]}" >/dev/null 2>&1 || return 1

}
cmd_notify () {

    source <(parse "$@" -- \
        platform:list platforms:list status title message \
        token chat telegram_token telegram_chat \
        slack_webhook discord_webhook webhook_url webhook \
        retries:int=3 delay:float=1 timeout:float=10 max_time:float=20 retry_max_time:float=60 \
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
    elif (( ${#platforms[@]} )); then
        plats+=( "${platforms[@]}" )
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
