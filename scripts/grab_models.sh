#!/usr/bin/env sh
set -eu

URL="https://drive.google.com/drive/folders/1cMiW4TzBQ2uBHJBOoDc1ZbFlT1CIj5_f"
PROJECT_ROOT="/app"
VENV_ROOT="${PROJECT_ROOT}/.venv"
ACTIVATE="${VENV_ROOT}/bin/activate"

warn() {
    msg="$1"

    echo "${msg}" > /dev/stderr
}

check_if_container() {
    if [ -d /app ]
    then
        return 0
    else
        warn "ERROR: This script is intended to be run inside of a docker container!"
        warn "       Please DO NOT run this inside of a native environment!"

        return 1
    fi
}

prepare() {
    if [ ! -d /app/.venv ]
    then
        python3 -m venv /app/.venv
        ( \
            source "${ACTIVATE}" \
            && pip install -U pip \
            && pip install gdown \
        )
    fi

    return $?
}

download() {
    [ -d "${PROJECT_ROOT}/models" ] || mkdir -v "${PROJECT_ROOT}/models"
    ( \
        source "${ACTIVATE}"
        gdown --folder "${URL}" -O "${PROJECT_ROOT}/models" \
    )
}

# Main
check_if_container
prepare
download
