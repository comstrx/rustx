#!/usr/bin/env bash

CURRENT_DIR=""
PLAY_SOUND_PID=""
PLAY_SOUND_PG=0

userhost_short () {

    local u="$(id -un 2>/dev/null || printf '%s' "${USER-unknown}")"
    local h="$(hostname 2>/dev/null || printf '%s' "${HOSTNAME-localhost}")"

    h="${h%%.*}"
    printf '%s@%s' "${u}" "${h}"

}
set_prompt () {

    local path="${1-}"

    if [[ -t 1 && -n "${TERM-}" && "${TERM}" != "dumb" && -z "${NO_COLOR-}" ]]; then
        CURRENT_DIR="\e[32m$(userhost_short)\e[0m:\e[34m${path}\e[0m\e[37m$ \e[0m"
    else
        CURRENT_DIR="$(userhost_short):${path}$ "
    fi

}
play_sound () {

    local file="${1-}" mode="${2-}" win_path="" cmd=""

    stop_sound

    [[ -n "${file}" ]] || return 0
    [[ -f "${file}" ]] || return 0

    [[ -n "${NO_SOUND-}" || "${SOUND-1}" == "0" ]] && return 0
    [[ -t 1 && -n "${TERM-}" && "${TERM}" != "dumb" ]] || return 0

    [[ "${mode}" == "loop" ]] || mode="once"

    if command -v afplay >/dev/null 2>&1; then

        if [[ "${mode}" == "loop" ]]; then

            if command -v setsid >/dev/null 2>&1; then
                setsid bash -c 'while :; do afplay "$1" >/dev/null 2>&1 || exit 0; done' _ "${file}" & PLAY_SOUND_PID=$!
                PLAY_SOUND_PG=1
            else
                ( while :; do afplay "${file}" >/dev/null 2>&1 || exit 0; done ) & PLAY_SOUND_PID=$!
                PLAY_SOUND_PG=0
            fi

        else

            afplay "${file}" >/dev/null 2>&1 & PLAY_SOUND_PID=$!
            PLAY_SOUND_PG=0

        fi

        return 0

    fi
    if command -v ffplay >/dev/null 2>&1; then

        if [[ "${mode}" == "loop" ]]; then

            if command -v setsid >/dev/null 2>&1; then
                setsid ffplay -nodisp -loglevel error -loop -1 "${file}" >/dev/null 2>&1 & PLAY_SOUND_PID=$!
                PLAY_SOUND_PG=1
            else
                ffplay -nodisp -loglevel error -loop -1 "${file}" >/dev/null 2>&1 & PLAY_SOUND_PID=$!
                PLAY_SOUND_PG=0
            fi

        else

            ffplay -nodisp -autoexit -loglevel error "${file}" >/dev/null 2>&1 & PLAY_SOUND_PID=$!
            PLAY_SOUND_PG=0

        fi

        return 0

    fi
    if command -v paplay >/dev/null 2>&1; then

        if [[ "${mode}" == "loop" ]]; then

            if command -v setsid >/dev/null 2>&1; then
                setsid bash -c 'while :; do paplay "$1" >/dev/null 2>&1 || exit 0; done' _ "${file}" & PLAY_SOUND_PID=$!
                PLAY_SOUND_PG=1
            else
                ( while :; do paplay "${file}" >/dev/null 2>&1 || exit 0; done ) & PLAY_SOUND_PID=$!
                PLAY_SOUND_PG=0
            fi

        else

            paplay "${file}" >/dev/null 2>&1 & PLAY_SOUND_PID=$!
            PLAY_SOUND_PG=0

        fi

        return 0

    fi
    if command -v aplay >/dev/null 2>&1; then

        if [[ "${mode}" == "loop" ]]; then

            if command -v setsid >/dev/null 2>&1; then
                setsid bash -c 'while :; do aplay -q "$1" >/dev/null 2>&1 || exit 0; done' _ "${file}" & PLAY_SOUND_PID=$!
                PLAY_SOUND_PG=1
            else
                ( while :; do aplay -q "${file}" >/dev/null 2>&1 || exit 0; done ) & PLAY_SOUND_PID=$!
                PLAY_SOUND_PG=0
            fi

        else

            aplay -q "${file}" >/dev/null 2>&1 & PLAY_SOUND_PID=$!
            PLAY_SOUND_PG=0

        fi

        return 0

    fi
    if command -v powershell.exe >/dev/null 2>&1; then

        win_path="${file}"
        command -v wslpath >/dev/null 2>&1 && win_path="$(wslpath -w "${file}" 2>/dev/null || printf '%s' "${file}")"
        win_path="${win_path//\'/\'\'}"

        if [[ "${mode}" == "loop" ]]; then cmd="\$p='${win_path}'; \$sp=New-Object Media.SoundPlayer \$p; while(\$true){ \$sp.PlaySync() }"
        else cmd="(New-Object Media.SoundPlayer '${win_path}').PlaySync()"
        fi

        powershell.exe -NoProfile -NonInteractive -Command "${cmd}" >/dev/null 2>&1 & PLAY_SOUND_PID=$!
        PLAY_SOUND_PG=0

        return 0

    fi

    return 0

}
stop_sound () {

    local pid="${PLAY_SOUND_PID-}"

    [[ -n "${pid}" ]] || return 0

    if command -v taskkill.exe >/dev/null 2>&1; then
        taskkill.exe //T //F //PID "${pid}" >/dev/null 2>&1 || true
    fi

    kill -TERM "${pid}" >/dev/null 2>&1 || true
    kill -TERM -- "-${pid}" >/dev/null 2>&1 || true

    if command -v pkill >/dev/null 2>&1; then
        pkill -TERM -P "${pid}" >/dev/null 2>&1 || true
    fi

    for _ in 1 2 3 4 5 6 7 8; do
        kill -0 "${pid}" >/dev/null 2>&1 || break
        sleep 0.03
    done

    kill -KILL "${pid}" >/dev/null 2>&1 || true
    kill -KILL -- "-${pid}" >/dev/null 2>&1 || true

    if command -v pkill >/dev/null 2>&1; then
        pkill -KILL -P "${pid}" >/dev/null 2>&1 || true
    fi

    wait "${pid}" 2>/dev/null || true

    PLAY_SOUND_PID=""
    PLAY_SOUND_PG=0

}
sound_traps () {

    trap 'stop_sound; exit 130' INT
    trap 'stop_sound' TERM
    trap 'stop_sound' EXIT

}
type_line () {

    local s="${1-}" type="${2-cmd}" sound="${3:-cmd}" mode="${4-loop}" delay="" i=0

    [[ "${type}" != "say" ]] && printf '%b' "${CURRENT_DIR}"

    sleep 1
    delay="$(awk -v r=22 'BEGIN{print 1/r}')"

    play_sound "${ROOT_DIR}/scripts/assets/${sound}" "${mode}"

    while (( i < ${#s} )); do
        printf '%s' "${s:i:1}"
        sleep "${delay}"
        i=$(( i + 1 ))
    done

    stop_sound
    printf '\n'

}

run_typed () {

    local rc=0 shown=""
    (( $# )) || return 0

    if (( $# == 1 )); then
        shown="${1}"
    else
        shown="$(printf '%q ' "$@")"
        shown="${shown% }"
    fi

    type_line "${shown}" cmd cmd.wav

    if (( $# == 1 )); then
        if bash -lc "${1}"; then :
        else rc=$?
        fi
    else
        if "$@"; then :
        else rc=$?
        fi
    fi

}
run_clear () {

    type_line clear cmd cmd.wav
    clear

}
run_say () {

    local new_line="${2:-1}"
    (( new_line )) && type_line && printf '\n'

    type_line "          👉  $1" say say.wav
    printf '\n'

}

cmd_cinema_help () {

    info_ln "Cinema :\n"

    printf '    %s\n' \
        "cinema-show                * Guided tour: boot a demo workspace and showcase commands" \
        "extract-gif                * Extract GIF(s) from video in high quality with support for start/duration/fps/width/loop." \
        ''

}
cmd_extract_gif () {

    ensure_pkg ffmpeg
    source <(parse "$@" -- :path out :start="00:00:00" :duration:float=60 :fps:float=20 :width:int=720 :speed:float=1 :quality=normal loop:bool)

    [[ -f "${path}" ]] || die "extract-gif: file not found: ${path}" 2
    [[ -n "${out}" ]] || out="${path%.*}.gif"

    local inv="$(awk -v s="${speed}" 'BEGIN{ if (s <= 0) s = 1; printf "%.10f", 1/s }')"
    local vf="" ff_loop="-1"

    (( loop )) && ff_loop="0"

    case "${quality}" in
        high) vf="setpts=${inv}*PTS,fps=${fps},scale=${width}:-1:flags=lanczos,split[s0][s1];[s0]palettegen=stats_mode=full[p];[s1][p]paletteuse=dither=bayer:bayer_scale=5" ;;
        *) vf="setpts=${inv}*PTS,fps=${fps},scale=${width}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" ;;
    esac

    ffmpeg -hide_banner -loglevel error -nostdin -y -ss "${start}" -t "${duration}" -i "${path}" -vf "${vf}" -loop "${ff_loop}" "${out}"

}
cmd_cinema_show () {

    sound_traps

    source <(parse "$@" -- \
        :tmp_dir="/tmp/rustx-demo" \
        :dir_name="RustxDemo" \
        :user="comstrx" \
        :repo="demo" \
        :src_repo="git@github.com:comstrx/rustx.git" \
        :installer="install.sh" \
    )

    rm -rf -- "${tmp_dir}/${dir_name}" 2>/dev/null || true
    cd -- "${tmp_dir}" || return 1
    run_clear
    sleep 1
    set_prompt ""
    run_typed "mkdir -p ${tmp_dir}"
    run_typed "cd ${tmp_dir}"
    set_prompt "${tmp_dir}"

    run_clear
    run_say "Hi, Rustaceans 🫡"
    run_say "Prepare your coffee and enjoy the show ☕" 0
    run_say "Let's get started and see the amazing rustx commands 😎" 0
    sleep 1

    cd -- "${tmp_dir}/${dir_name}" || return 1
    run_clear
    run_say "Boot a clean workspace."
    run_typed "mkdir -p ${dir_name}"
    run_typed "cd ${dir_name}"
    set_prompt "${tmp_dir}/${dir_name}"

    run_clear
    run_say "clone the template into this folder."
    run_typed "git clone ${src_repo} ."

    run_clear
    run_say "Install an isolated rustx demo launcher and configure its repository metadata."
    run_typed "bash ${installer} --alias rustx-demo --name demo --description \"this is my demo crate\" --user \"${user}\" --repo \"${repo}\""
    run_typed "which rustx-demo"

    run_clear
    run_say "Create the GitHub repo and wire this workspace for real pushes/releases."
    run_typed "rustx-demo init ${user}/${repo}"
    run_say "Check current remote status, atuh ..."
    run_typed "rustx-demo remote"
    run_say "Creating secrets and environmental variables."
    run_typed "mv .secret.example .secret"
    run_say "Fill in the secret values in the .secret file."
    run_say "Now sync secrets from your local file into GitHub."
    run_typed "rustx-demo sync-secrets"

    run_clear
    run_say "Show the command surface: one CLI to rule the whole workspace."
    run_typed "rustx-demo --help"

    run_clear
    run_say "Help pages for the main pillars: lint/ci/doctor (fast discovery without reading docs)."
    run_typed "rustx-demo cinema-help"
    run_typed "rustx-demo notify-help"
    run_typed "rustx-demo git-help"
    run_typed "rustx-demo github-help"
    run_typed "rustx-demo crate-help"
    run_typed "rustx-demo lint-help"
    run_typed "rustx-demo perf-help"
    run_typed "rustx-demo safety-help"
    run_typed "rustx-demo doctor-help"
    run_typed "rustx-demo meta-help"
    run_typed "rustx-demo ci-help"

    run_clear
    run_say "Auto help for any command: (args/options/flags)."
    run_typed "rustx-demo help sync-secrets"
    run_typed "rustx-demo help cinema-show"
    run_typed "rustx-demo help publish"
    run_typed "rustx-demo help yank"
    run_typed "rustx-demo help notify"
    run_typed "rustx-demo help push"
    run_typed "rustx-demo help samply"
    run_typed "rustx-demo help fuzz"
    run_typed "rustx-demo help doctor"

    run_clear
    run_say "See source code + file path (and line numbers) for any command."
    run_typed "rustx-demo source help"
    run_typed "rustx-demo source add-var"
    run_typed "rustx-demo source add-secret"
    run_typed "rustx-demo source ensure"
    run_typed "rustx-demo source ci-local"

    run_clear
    run_say "Diagnose environment: OS + Bash + Rust + Git + required tools (catch problems before CI screams)."
    run_typed "rustx-demo doctor"
    run_clear
    run_say "Ensure tooling: installs/validates cargo tools and utilities (autopilot mode)."
    run_typed "rustx-demo ensure"

    run_clear
    run_say "Toolchain matrix: show and run stable/nightly/MSRV and inspect active toolchain."
    run_typed "rustx-demo version"
    run_typed "rustx-demo stable"
    run_typed "rustx-demo nightly"
    run_typed "rustx-demo msrv"
    run_typed "rustx-demo active"

    run_clear
    run_say "Create a new crate named 'hi', a publishable crate and a private/internal one, then inspect manifests."
    run_typed "rustx-demo new hi"
    run_typed "cat crates/hi/Cargo.toml"
    run_typed $'cat > crates/hi/src/lib.rs <<\'RS\'\n//! Hi crate Doc.\n\npub struct Hi;\n\nimpl Hi {\n    #[must_use]\n    pub const fn run() -> &\'static str {\n        "Iam Hi Crate"\n    }\n}\nRS\n'

    run_clear
    run_say "Workspace metadata: list package names, then only publishable crates (release safety)."
    run_typed "rustx-demo meta --names"
    run_typed "rustx-demo meta --names --only-publishable"
    run_typed "rustx-demo can-publish"
    run_typed "rustx-demo tree"
    run_typed "rustx-demo tree-files"

    run_clear
    run_say "Read-only quality gates (checks): verify formatting, audits, TOML, and prettier without changing files."
    run_typed "rustx-demo fmt-check"
    run_typed "rustx-demo audit-check"
    run_typed "rustx-demo taplo-check"
    run_typed "rustx-demo prettier-check"

    run_clear
    run_say "Auto-fix gates: apply consistent formatting + repair common policy issues (make it green fast)."
    run_typed "rustx-demo ws-fix"
    run_typed "rustx-demo fmt-fix"
    run_typed "rustx-demo audit-fix"
    run_typed "rustx-demo taplo-fix"
    run_typed "rustx-demo prettier-fix"

    run_clear
    run_say "Supply-chain verification: vet policy check (dependency trust gate)."
    run_typed "rustx-demo vet-check"
    run_say 'Trust the "best" baseline set (project-defined trust presets).'
    run_typed "rustx-demo vet-trust-best"
    run_say 'Import the "best" audit set (project-defined import presets).'
    run_typed "rustx-demo vet-import-best"
    run_say "Supply-chain verification: vet policy check (dependency trust gate)."
    run_typed "rustx-demo vet-check"

    run_clear
    run_say "Core build/run suite: check, test, and documentation checks."
    run_typed "rustx-demo check"
    run_typed "rustx-demo test"
    run_typed "rustx-demo doc-check"
    run_typed "rustx-demo doc-test"
    run_typed "rustx-demo doc-clean"

    run_clear
    run_say "UB & chaos testing: fuzz quickly, then sanitizer + miri (catch dragons)."
    run_typed "rustx-demo fuzz --timeout 5"
    run_typed "rustx-demo sanitizer"
    run_typed "rustx-demo miri"

    run_clear
    run_say "Clippy gate: strict linting to keep the codebase honest."
    run_typed "rustx-demo clippy"
    run_clear
    run_say "Coverage output: produce lcov report artifact for CI/codecov."
    run_typed "rustx-demo coverage --out out/lcov.info"
    run_clear
    run_say "Size gate: produce bloat report artifact (binary weight accountability)."
    run_typed "rustx-demo bloat --out out/bloat.info"

    run_clear
    run_say "Full local CI simulation: run the whole pipeline before pushing (gatekeeper mode)."
    run_typed "rustx-demo ci-local"
    run_clear
    run_say "Release flow: tag + changelog + push (force only for demo purposes)."
    run_typed "rustx-demo push --release --changelog --force"
    run_typed "cat CHANGELOG.md"

    run_clear
    run_say "Goodbye, Rustaceans 😎"

    sleep 5

}
