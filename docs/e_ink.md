# E-INK

Eigenschaften:

- Kein/Geringer Stromverbrauch im Standby

- Keine Hintergrundbeleuchtung

- Hoher Kontrast

- Extrem hohe Betrachtungswinkel

## Mögliche Kaufoptionen

[Amazon Link](https://www.amazon.de/4-2inch-Display-Module-Resolution-Two-Color/dp/B07Q6VBF89/ref=pd_sbs_sccl_2_4/260-4331787-8265318?pd_rd_w=4mKxU&pf_rd_p=5cb0d797-c494-4ef9-9229-f54873a5d2fa&pf_rd_r=71MZEBGFDC60BMAR7F6V&pd_rd_r=61f4d557-80b3-4448-812a-56c8411dd62b&pd_rd_wg=MKTeH&pd_rd_i=B07Q6VBF89&psc=1)

[EXP Tech E-INK + Raspi Adapter](https://www.exp-tech.de/displays/e-paper-e-ink/8874/waveshare-600x448-5.83-e-ink-display-hat?c=1424)


[ESP8266 E-INK Driver Board](https://eckstein-shop.de/WaveshareUniversale-PaperRawPanelDriverBoardESP8266WiFiWireless?googlede=1&gclid=CjwKCAjw4ayUBhA4EiwATWyBrlbHYgcY60LKnQwP5-yufKuXcEY5JNBKk6Az1PZE5ae4DerI4xzAFBoCFUgQAvD_BwE) + [E-INK Raw Display](https://www.exp-tech.de/displays/e-paper-e-ink/8885/waveshare-4.2-e-ink-raw-display-400x300?c=1424)

# SPI

SPI = Serial Peripheral Interface

- Vollduplexfähig
- Max 20 Mbps Übertragungsrate

### SPI Modi

Modi gibt an anhand von welchen Werten(fallende/steigende Flanke) Master und Slave sich synchronisieren.

(siehe [Abschnitt Taktpolarität und Taktphase; Bild 3-5](Taktpolarität und Taktphase))


### SPI Pin Layout

Minimum 4Pin Verbindung: 

- SLC = Serial Clock

- POCI/MISO = Master Input/ Slave Output

- PICO/MOSI = Master Output/ Slave Input

- CS = Chip select; SS = Slace select


### SPI Konfiguraton

Benötigte Parameter:

- Taktflanke

- Wortlänge

- Übertragungsart(Bitwertigkeit)

- Übertragungsfrequenz


### Erklärungen

[Edi's Techlab, erklärt Konfiguration und Modi + Video](https://edistechlab.com/wie-funktioniert-spi/?v=3a52f3c22ed6)

[SPI in C; Educational Paper](https://www.rpi.edu/dept/ecse/mps/Coding_SPI_sw.pdf)

[Mikrocontroller.net](https://www.mikrocontroller.net/articles/Serial_Peripheral_Interface)

[Kunbus; Kurztext zu SPI](https://www.kunbus.de/die-spi-schnittstelle.html)


## Beispiel Implentierungen:

[Implementierung in C ohne zusatzbibliotheken](https://www.maximintegrated.com/en/design/technical-documents/app-notes/4/4184.html)

[C SPI Library](http://onioniot.github.io/wiki/Documentation/Libraries/SPI-C-Library.html)

[Rust SPI Implementierung](https://crates.io/crates/display-interface-spi)

[Rust SPI + DMA Mockup](https://github.com/stm32-rs/stm32f1xx-hal/blob/master/examples/spi-dma.rs)

