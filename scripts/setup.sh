#!/bin/bash

SCRIPT_DIRECTORY="$(cd -- "$(dirname "$0")" > /dev/null 2>&1 ; pwd -P)"
cd $SCRIPT_DIRECTORY

hostname=$1

scp ../digital-fax.service pi@$hostname:~
scp ../management/arm-binary pi@$hostname:~/management

ssh pi@$hostname << EOF
openssl genpkey -algorithm ed25519 -out digital_fax_private.pem
sudo systemctl stop digital-fax
sudo mv ./digital-fax.service /etc/systemd/system/digital-fax.service
sudo systemctl enable digital-fax
sudo systemctl start digital-fax
EOF