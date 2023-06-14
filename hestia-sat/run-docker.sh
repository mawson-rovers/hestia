#!/bin/bash

set -e

docker run -it -v "$PWD":/usr/cubeos \
    -v /run/host-services/ssh-auth.sock:/run/host-services/ssh-auth.sock:ro \
    -e SSH_AUTH_SOCK="/run/host-services/ssh-auth.sock" \
    -w /usr/cubeos cubeos-dev bash

