PASTEBIN_URL_BASE=https://pastebin.com/raw
QUAD=yUd2WLsE

OUT_DIR=models

warn() {
    echo $@ > /dev/error
}

quad() {
    curl -L -o "${OUT_DIR}/quad.obj" "${PASTEBIN_URL_BASE}/${QUAD}"

    return $?
}

if [ $# -eq 0 ]
then
    warn "Please specify which obj file you want to download."
    warn "  Available:"
    warn "      > quad"
    false
fi

case "$1" in
    "quad")
        quad
        ;;
    *)
        warn "obj file not available:"
        warn "Please specify which obj file you want to download."
        warn "  Available:"
        warn "      > quad"
        false
        ;;
esac
