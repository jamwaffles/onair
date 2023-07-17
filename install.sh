#!/usr/bin/env bash

set -ex

cargo build --release

sudo cp ./target/release/onair /usr/local/bin/onair
sudo cp ./onair.service /etc/systemd/user

systemctl --user daemon-reload

systemctl --user enable onair.service
systemctl --user start onair.service
