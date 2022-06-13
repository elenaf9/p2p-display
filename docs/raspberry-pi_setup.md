# Running the Rust P2P-network on a Raspberry PI

Setup:
- Raspberry Pi: Raspberry Pi 3
- MicroSD
- Local password protected Wifi

## Configuring headless Raspberry Pi

1. Write Raspberry Pi OS to micro SD
   - Using <https://www.raspberrypi.com/software/>
   - Select _RASPBERRY PI OS LITE (32-BIT)_
   - Once it is done, two partition exists on the SD: _boot_ and _rootfs_
2. Configure headless setup
   - Enable ssh: Create an empty file `ssh` in the boot folder
   - Configure Wifi connection: create file `wpa_supplicant.conf` in boot folder:
    ```
    ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev
    update_config=1
    country=DE

    network={
        ssid="<wifi-ssid>"
        psk="wifi-password"
    }
    ```
   - Add own ssh pub key so that no password is needed:
     - In rootfs folder: create folder `/home/pi/.ssh`
     - Create file `home/pi/.ssh/authorized_keys`
     - Write own ssh public key into the file
3. Insert micro SD to Pi, start Pi
4. Connect to Pi via SSH: `ssh pi@raspberrypi.local` -> should not need a password since we added our pub-key

## Running the p2p network

On the Raspberrpi Pi:
1. Install Rust: `curl https://sh.rustup.rs -sSf | sh`
2. Setup cargo env: `source $HOME/.cargo/env`
3. Install Protocol Buffer Compiler: `sudo apt install -y protobuf-compiler`
4. Copy the zipped `p2p-network` folder to pi. On the host machine:
   - Run `cargo clean` before creating the zip so that the build files are not included
   - Create zip archive of `p2p-network`
   - Copy the zip via ssh to the pi: `scp p2p-network.zip pi@raspberrypi.local:~/p2p-network.zip`
5. Unzip the folder on the pi: `unzip p2p-network.zip`
6. Build and run the binary: `cd p2p-network && cargo run`
7. Run another peer in the same network, e.g. on the host machines -> Peers should be able to communicate
