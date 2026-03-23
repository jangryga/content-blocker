#!/bin/bash

set -e

SERVICE="Wi-Fi"
MODE=$1

if [[ "$MODE" != "ON" && "$MODE" != "OFF" ]]; then
    echo "Error: MODE must be 'ON' or 'OFF'"
    exit 1
fi

networksetup -setwebproxy "$SERVICE" 127.0.0.1 8080
networksetup -setsecurewebproxy "$SERVICE" 127.0.0.1 8080

if [[ "$MODE" == "ON" ]]; then
    networksetup -setwebproxystate "$SERVICE" on
    networksetup -setsecurewebproxystate "$SERVICE" on
    echo "Proxy enabled"
else
    networksetup -setwebproxystate "$SERVICE" off
    networksetup -setsecurewebproxystate "$SERVICE" off
    echo "Proxy disabled"
fi
