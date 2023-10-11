#!/bin/bash

set -e

tailwindcss -i static/tailwind-input.css -o static/tailwind-output.css --watch=always &
python -m http.server 5002
