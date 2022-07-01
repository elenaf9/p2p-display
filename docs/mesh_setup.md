Access Bridge access point network

1. Power up Raspi 4 ( N. 1 Bridge) with usb dongle
2. Look for SSID raspimesh in wifi network
3. connect with password "swp123swp123" (YOU HAVE TO USE THE " ", DON'T ASK WHY)

Access Bridge over ssh

1. > ssh pi@raspberrypi1.local
2. username: pi
3. password: swp123

Access gateway over ssh

1. > ssh pi@raspberrypi2.local
2. username: pi
3. password: swp123

access node over ssh

1. > ssh pi@raspberrypi3.local
2. username: pi
3. password: swp123

The mesh network takes some time to boot.

The Gateway gives out the 192. addresses to the nodes.

To test if a node can see its neighbours

> sudo batctl n

And you should see the mac addresses of the neighbours

It's possible that the last command shows you their names (through manual configuration in each node) but I was too lazy





-----------------------------------------------------------------------------
Follow Tutorial on:

https://github.com/binnes/WiFiMeshRaspberryPi/blob/master/README.md

part 2 bridge access point is total bullshit though
