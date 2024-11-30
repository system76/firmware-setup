# System76 Firmware Setup

firmware-setup is a UEFI driver that implements the user interface for
System76 Open Firmware. It is basic by design, with its only functionality
being selecting the boot device and changing the boot order.

## Testing

As a basic test, the driver can be loaded in QEMU to use the UI.

```
make qemu
```

In QEMU:

```
Shell> fs0:
FS0:\> load release\system76_firmware_setup.efi
FS0:\> exit
```
