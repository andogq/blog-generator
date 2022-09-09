#!/bin/bash

tmux split-window -h

tmux send-keys -t 0 "cd webapp; npm run dev" Enter
tmux send-keys -t 1 "cd worker; miniflare -e ../.env.dev" Enter
