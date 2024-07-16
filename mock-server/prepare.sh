#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"
rm -rf ./public
mkdir -p ./public/{ufc,obfuscated,bandit}/api/flag-config/v1/
ln -s ../../../../../../sdk-test-data/ufc/flags-v1.json ./public/ufc/api/flag-config/v1/config
ln -s ../../../../../../sdk-test-data/ufc/flags-v1-obfuscated.json ./public/obfuscated/api/flag-config/v1/config
ln -s ../../../../../../sdk-test-data/ufc/bandit-flags-v1.json ./public/bandit/api/flag-config/v1/config
ln -s ../../../../../../sdk-test-data/ufc/bandit-models-v1.json ./public/bandit/api/flag-config/v1/bandits
