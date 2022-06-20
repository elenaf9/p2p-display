# Display Component

## Hardware Pinout

E-INK        =>    Raspberry Pi

VCC    ->    3.3

GND    ->    GND

DIN    ->    10(SPI0_MOSI)

CLK    ->    11(SPI0_SCK)

CS     ->    8(SPI0_CS0)

DC     ->    25

RST    ->    17

BUSY   ->    24

## Installation

### Package Vorraussetzungen

```console
sudo apt-get install libc6-dev libstdc++6 libcap2 libcap-dev python3-pip python3-pil python3-numpy
sudo pip3 install RPi.GPIO
sudo pip3 install spidev
sudo reboot
```

### Driver Files

```console
wget http://www.airspayce.com/mikem/bcm2835/bcm2835-1.71.tar.gz
tar zxvf bcm2835-1.71.tar.gz
cd bcm2835-1.71
sudo ./configure && sudo make && sudo make check && sudo make install
wget https://project-downloads.drogon.net/wiringpi-latest.deb
sudo apt install ./wiringpi-latest.deb -y
gpio -v
sudo reboot
```

## Notes

* Schriftart gefixt [0 = Dunkle Schrift, weißer Hintergrund; 1 = weiße Schrift, dunkler Hintergrund]

* Schriftgröße funktioniert nur auf 12, keine weiteren größen Definiert
