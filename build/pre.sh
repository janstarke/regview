#!/bin/bash

echo "updating package metadata"
echo "============================================"
if [ -f /etc/alpine-release ]; then
    apk update
else 
    apt-get update
fi
echo "============================================"

echo "installing missing packages"
if [ -f /etc/alpine-release ]; then
    apk add clang-static ncurses-dev
else
    apt-get install -y libncurses-dev
fi
echo "============================================"