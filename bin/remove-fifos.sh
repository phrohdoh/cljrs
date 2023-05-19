#!/usr/bin/env sh

# relies on .envrc being sourced (`direnv allow`)

rm "${CLJRS_STDIN_FIFO_PATH}" && echo "removed ${CLJRS_STDIN_FIFO_PATH}"
