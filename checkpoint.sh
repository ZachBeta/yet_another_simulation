#!/bin/bash

# git add, git commit, use all args as commit message else "Checkpoint"
git add .
if [ -z "$1" ]; then
    git commit -am "Checkpoint"
else
    git commit -am "$*"
fi
