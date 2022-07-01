CONNECTING

Access Bridge access point network

1. Power up Raspi 4 (pi1 Bridge) with usb dongle (WITH DONGLE!!!!!!!!!!!!!!!!!!!!!)
2. Look for SSID raspimesh in wifi network
3. connect with password: swp123swp123

Needs some time to connect



Access Bridge over ssh

1. > ssh pi@raspberrypi1.local
2. username: pi
3. password: swp123



Access gateway over ssh (you can plug a usb wifi dongle in the gateway, more information below)

1. > ssh pi@raspberrypi2.local
2. username: pi
3. password: swp123

If gateway not accessible try accessing it over the usb dongle (instructions below) via a same network (smartphone hotspot/ laptop needs to be on the same network) You have to connect the bridge via ethernet though)


access node over ssh

1. > ssh pi@raspberrypi3.local
(raspberrypi_ change the number to the node)
2. username: pi
3. password: swp123

The mesh network takes some time to boot.

The Gateway gives out the 192. addresses to the nodes.

---------------------------------------------------------------------------------------------------------------------------
TESTING/VERIFYING NODES

To test if a node can see its neighbours

> sudo batctl n

And you should see the mac addresses of the neighbours

Run ifconfig on all the nodes and check that:

On Bridge (pi1):

br0 has a 192.x.x.x address


On Gateway (pi2)

bat0 has a 192.x.x.x address
-------------------------------------------------------------------------------------------------------------------
CONNECTING GATEWAY USB DONGLE TO NEW NETWORK

Go to

>sudo nano /etc/wpa_supplicant/wpa_supplicant.conf

>Change ssid and psk field

(maybe have to reboot i don't know)

-----------------------------------------------------------------------------------------------------------------------------
ADDING A NODE:


Follow part1:

https://github.com/binnes/WiFiMeshRaspberryPi/blob/master/README.md

Sometimes you need to first execute

>sudo apt-get install vlc-bin

before updating and upgrading the pi (As in the github guide)

The reboot and look with ">sudo batctl n" if a neighbour can find it (the mac dress will show up)

--------------------------------------------------------------------------------------------------------------------------
UPDATE NODE NAME IN NEIGHBOUR BATCTL N COMMAND (NOT IMPORTANT)

If you add a new node, only the mac adress will show up in batcl. To add a name to the new device's mac address:

1. Find out the new node's mac adress by calling the following command on an already existing node
> sudo batctl n

and copythe device's mac address

2. execute 

> sudo nano /etc/bat-hosts

and copy 
cc:b2:55:55:27:da bi-raspimesh01
b8:27:eb:ab:2f:3f bi-raspimesh02
e4:5f:01:84:f6:e0 bi-raspimesh03

and add new mac address with name in the in the line below





-----------------------------------------------------------------------------
Follow Tutorial on:

https://github.com/binnes/WiFiMeshRaspberryPi/blob/master/README.md

part 2 bridge access point is total bullshit though
