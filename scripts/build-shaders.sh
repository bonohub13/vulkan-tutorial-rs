#!/usr/bin/env sh
set -eu

SHADER_DIR="shaders"

warn() {
    echo $@ > /dev/stderr
}

dependency() {
    if [ "`which glslc`" = "" ]
    then
        warn "Following program is missing."
        warn "  > glslc"

        return 1
    fi

    return 0
}

compile() {
    file_name="$1"

    glslc "${file_name}" -o "${file_name}.spv"

    return $?
}

dependency
find "${SHADER_DIR}" -type f \
    | grep -v "\.spv$" \
    | while read f
do
    compile "$f"
done
