#!/usr/bin/env sh

# relies on .envrc being sourced (`direnv allow`)

mkfifo "${CLJRS_STDIN_FIFO_PATH}" > /dev/null && echo "created ${CLJRS_STDIN_FIFO_PATH}"
