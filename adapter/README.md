# SSL Adapter 

This file documents the setup for Cinch's SSL adapter

## Requirements

- You will need a machine or device running Linux to serve as the adapter (e.g., BeagleBone)

- You will need stunnel4 installed on your machine.

- You will need our modified version of usbredir (adapter/usbredir\_ssl):

  ```sh
  git submodule init
  git submodule update
  ```

- Below we refer to a BASEDIR environment variable; this can be, e.g., $HOME,
  /usr/local, etc.

## copy files into the right places and generate SSL keys

  ```sh
  mkdir -p $BASEDIR/usbredir/keys
  cp genkeys.sh $BASEDIR/usbredir/keys
  cp stunnel_config.{client,server} $BASEDIR/usbredir
  cd $BASEDIR/usbredir/keys
  ./genkeys.sh
  ```

You will need the client\_\* and ca\_cert.pem on the client (blue machine);
you will need the server\_\* and ca\_cert.pem on the server (adapter).

## Adapter's USB network connection

The SSL adapter needs to have a USB network gadget running and connected to the
red machine.

To configure the network gadget follow the steps 
[here](https://developer.ridgerun.com/wiki/index.php/How_to_use_USB_device_networking).

Connect the SSL adapter to the Red machine, and give the USB network device an IP
address on the same subnet at both ends. You should be able to ping from one
end of the connection to the other.

## usbredirserver on the SSL adapter

You have two options for the server. Either run a modified usbredirserver
binary with TLS support, or use stunnel to act as a proxy.

### usbredir\_ssl

You can then build the usbredirserver binary using:

  ```sh
  cd usbredir_ssl
  ./autogen.sh
  ./configure --prefix=$BASEDIR/usbredir
  make && make install
  ```

Now, make sure that you have the server\_\* and ca\_cert.pem files in $BASEDIR/usbredir/keys.

  ```sh
  cd $BASEDIR/usbredir
  sbin/usbredirserver -p 9999 -s XXXX:YYYY
  ```

### stunnel as a server

Instead, you can use stunnel and the system-wide usbredirserver (not patched for TLS support).

Make sure that the server\_\* and ca\_cert.pem files in $BASEDIR/usbredir/keys, then

  ```sh
  usbredirserver -p 9997 XXXX:YYYY        ## NOTE port 9997 here, not 9999!
  cd $BASEDIR/usbredir
  stunnel stunnel_config.server
  ```

## on the red machine

You need to expose a port to the blue machine from the red machine, and upon
receiving a connection on that port, make a new connection to the SSL adapter
at port 9999. (This will either be the TLS-enabled usbredirserver binary or the
stunnel server, which will relay the connection to usbredirserver).

socat lets you do this pretty simply:

  `socat tcp-listen:9999 tcp-connect:SSLADAPTER:9999`

## on the hypervisor

Finally, you will want to use stunnel in client mode to initiate the TLS
connection to the SSL adapter.

You will need to edit `stunnel_config.client` to give it the proper IP address
for the red machine (edit the line that says `connect=REDMACHINE:9999`).

You will also need the client\_\* and ca\_cert.pem files that you generated
above, and they should be in a directory called `keys`.

Once you have done the above,

  `stunnel stunnel_config.client`

The above will listen on `localhost:9998`.

## connect to the blue machine qemu instance

Finally, you can connect to the blue machine's qemu instance with something like

  ```
  qemu-system-x86_64 [your options go here] -readconfig /usr/share/doc/qemu-system-common/ich9-ehci-uhci.cfg -chardev socket,id=usbrdsock,host=127.0.0.1,port=9998 -device usb-redir,chardev=usbrdsock,id=usbrddev
  ```
The `-readconfig ...` option pulls in a file that defines some USB EHCI and
UHCI host controller devices so that the usb-redir device has somewhere
to connect. You can leave this out if you are already defining the USB host
hardware.
