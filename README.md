# Cinch
A system for interposing on USB transfers and several security modules.
See the paper [here] (https://www.usenix.org/system/files/conference/usenixsecurity16/sec16_paper_angel.pdf). 
Our current prototype works with Linux KVM hypervisor (it supports any guest OS as we describe in the paper).


# Compiling Cinch
Cinch is written in [Rust] (https://www.rust-lang.org) and compiles under the nightly compiler.
To choose the nightly compiler simply run:
```
$ rustup install nightly
$ rustup default nightly
```

To compile Cinch with debug symbols simply run: ``$ cargo build``. The resulting binary will be found in ``target/debug/cinch``.
To compile Cinch with all optimizations run: ``$ cargo build --release``. The resulting binary will be found in ``target/release/cinch``.

# Replicating the setup described in the paper

Our setup requires a machine with an IOMMU running Linux with QEMU/KVM installed.  

## Unbinding the USB host controller
The first step is to unbind the host controller from the Linux machine so that we can redirect it to the red machine VM.

Use the ``iommu_setup`` script found in ``scripts`` to unbind the machine's USB host controller.
Modify the values at the beginning of the script with those belonging to your particular host controller.

Then run: ``$ scripts/iommu_setup start``.

## Creating and configuring the Red machine VM
Create a new QEMU VM image and install Linux on it (we use Debian but any distribution works). Ensure tht the VM has a network 
connection to the Linux hypervisor (we did this by setting up a [network bridge] (http://www.linux-kvm.org/page/Networking)).

### Install usbredir on the Red machine
Now that the VM has network support, install [usbredir] (https://github.com/SPICE/usbredir) version 0.7.1 on the Red machine VM. 

In Debian, you can run: ``# apt-get install usbredir``. 
In Arch, you can run ``# pacman -S usbredir``.

Other distributions may have similar packages. Otherwise install it from source.

### Blacklist.conf
Depending on the Red machine OS that you install, its USB stack might be up to date and prevent many of the CVE exploits that
we test in the paper. To ensure that this is not the case (and let Cinch do the heavy lifting) blacklist all the relevant drivers.

In the red machine add the following to ``/etc/modprobe.d/blacklist.conf``

```
blacklist snd-usb-audio
blacklist ati_remote2
blacklist powermate
blacklist gtco
blacklist iowarrior
blacklist visor
blacklist mct_u232
blacklist cypress_m8
blacklist wacom
blacklist digi_acceleport
blacklist ims_pcu
blacklist cdc_acm
```
This will prevent any of these drivers from loading on the Red VM.


### Launch the Red VM with the USB host controller
Launch the red machine and bind the host controller to it using the IOMMU.
See the ``launch-red`` script found in ``scripts`` for the command line that we used to launch our red machine VM.

## Creating and configuring the Blue machine VM
For the blue machine VM you may use any OS supported by QEMU (e.g., Windows, \*BSD, Linux).
The blue machine does not require any special configuration.

### Launch the Blue VM
To launch the blue machine use the ``launch-blue`` script found in ``scripts``. 
It allows us to connect to the QEMU monitor using telnet which we leverage to hot-plug USB devices.

## Connecting devices from the Red to the Blue VM
First start Cinch with the default configuration (see next section for how to specify other configurations):

``$ target/release/cinch``  


### Exporting devices through Cinch
To export devices, first connect the device to the (physical) machine. The device should then appear on the Red VM.
Use the ``device_setup`` found in ``scripts`` to export this device (this is done from the Linux/KVM hypervisor).

```sh
$ scripts/device_setup ls # shows ids of available devices
$ scripts/device_setup export [ID] # unbinds device ID [ID] from Red VM and readies it for exporting to blue VM
$ scripts/device_setup start # hotplugs the device to the Blue VM through Cinch
```

### Exporting devices without Cinch
If you want to run this setup without Cinch, simply replace the ``CINCH_IP`` and ``CINCH_PORT`` to 
the IP of the Red VM and the port to 8000 (which is hardcoded in the script's export function).

# Configuring Cinch, Logging, and Signatures

## Cinch Configuration Format

Our current prototype has 8 options that can be specified.

*red_addr*: the IP:Port of the red machine. 
Cinch will connect to this address once the device has been 
[hotplugged](https://github.com/sga001/cinch#connecting-devices-from-the-red-to-the-blue-vm).

*cinch_addr*: The IP:Port of the Linux/KVM hypervisor instance running Cinch.

*log*: boolean flag stating whether to log all traffic or not.

*log_prefix*: path (and optional prefix name) for log files. For instance,
if ``/home/cinch-user/logs/trace`` is specified, all traffic will be stored in a file
called: ``trace-TIMESTAMP.log``. The timestamp has the format "day-month-year-hour-minute-second".

*checks_active*: boolean flag stating whether to perform compliance checks. If false, Cinch simply
acts as a transparent proxy.

*patch_active*: boolean flag stating whether to perform signature checks.

*patches*: absolute path to the directory holding signatures (we call them patches in the source code).
Each signature should be in a different JSON file.

*third_party_folder*: absolute path to the directory holding third party constraints. Each constraint
should be in a different JSON file.

Below is a sample config file.

```json
"red_addr": "192.168.1.100:8000",
"cinch_addr: "192.168.1.7:5555",
"log": true,
"log_prefix": "/home/cinch-user/log/experiment1",
"checks_active": true,
"patch_active": true,
"patches": "/home/cinch-user/cinch/signatures",
"third_party_folder": "/home/cinch-user/cinch/third-party-checks"
```

The IP addresess correspond to a local network between the Red VM and Cinch.

## Signature format

TODO

