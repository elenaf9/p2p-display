#!/bin/bash

SCRIPT_DIRECTORY="$(cd -- "$(dirname "$0")" > /dev/null 2>&1 ; pwd -P)"
cd $SCRIPT_DIRECTORY

hostname=$1
# sudo apt-get update && sudo apt-get upgrade -y

ssh pi@$hostname << EOF
sudo apt-get install -y batctl

tee ~/start-batman-adv.sh > /dev/null <<EOT
#!/bin/bash
# batman-adv interface to use
sudo batctl if add wlan0
sudo ifconfig bat0 mtu 1468

# Tell batman-adv this is a gateway client
sudo batctl gw_mode client

# Activates batman-adv interfaces
sudo ifconfig wlan0 up
sudo ifconfig bat0 up
EOT
chmod +x ~/start-batman-adv.sh

sudo tee /etc/network/interfaces.d/wlan0 > /dev/null <<EOT
auto wlan0
iface wlan0 inet manual
    wireless-channel 1
    wireless-essid call-code-mesh
    wireless-mode ad-hoc
EOT

echo 'batman-adv' | sudo tee --append /etc/modules
echo 'denyinterfaces wlan0' | sudo tee --append /etc/dhcpcd.conf

sudo tee /etc/rc.local > /dev/null <<EOT
#!/bin/sh -e
#
# rc.local
#
# This script is executed at the end of each multiuser runlevel.
# Make sure that the script will "exit 0" on success or any other
# value on error.
#
# In order to enable or disable this script just change the execution
# bits.
#
# By default this script does nothing.

# Print the IP address
_IP=$(hostname -I) || true
if [ "$_IP" ]; then
  printf "My IP address is %s\n" "$_IP"
fi

/home/pi/start-batman-adv.sh &

exit 0
EOT
EOF