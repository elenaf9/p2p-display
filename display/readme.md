# Installation

## Hardware Pinout

OLED        =>    Raspberry Pi
* VCC    ->    3.3
* GND    ->    GND
* DIN    ->    MOSI
* CLK    ->    SCLK
* CS     ->    24 (Physical, BCM: CE0, 8)
* D/C    ->    36 (Physical, BCM: 16)
* RES    ->    35 (Physical, BCM: 19)

## Installation

### Package Vorraussetzungen

```console
sudo apt-get install libc6-dev
``` 
```console
sudo apt-get install libstdc++6 
```
```console
sudo apt-get install libcap2 libcap-dev

```
```console
sudo adduser $USER kmem

```
```console
echo 'SUBSYSTEM=="mem", KERNEL=="mem", GROUP="kmem", MODE="0660"' | sudo tee /etc/udev/rules.d/98-mem.rules

```
```console
sudo reboot
```

### Driver Files
```console
wget http://www.airspayce.com/mikem/bcm2835/bcm2835-1.71.tar.gz
```
```console
tar zxvf bcm2835-1.71.tar.gz
```
```console
cd bcm2835-1.71
```
```console
./configure
```
```console
make
```
```console
sudo make check
```
```console
sudo make install
```
## Neue Binäry erstellen
```console
make
```

## Notes

* Schriftart gefixt [0 = Dunkle Schrift, weißer Hintergrund; 1 = weiße Schrift, dunkler Hintergrund]

* Schriftgröße funktioniert nur auf 12, keine weiteren größen Definiert
