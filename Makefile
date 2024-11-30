# SPDX-License-Identifier: GPL-3.0-only

TARGET = x86_64-unknown-uefi
QEMU = qemu-system-x86_64
OVMF = /usr/share/OVMF

.PHONY: build
build:
	cargo build --release

.PHONY: clean
clean:
	cargo clean

.PHONY: qemu
qemu: build $(OVMF)/OVMF_VARS.fd $(OVMF)/OVMF_CODE.fd
	cp $(OVMF)/OVMF_CODE.fd target/
	cp $(OVMF)/OVMF_VARS.fd target/
	$(QEMU) -enable-kvm -M q35 -m 1024 -vga std \
		-chardev stdio,mux=on,id=debug \
		-device isa-serial,index=2,chardev=debug \
		-device isa-debugcon,iobase=0x402,chardev=debug \
		-drive if=pflash,format=raw,readonly=on,file=target/OVMF_CODE.fd \
		-drive if=pflash,format=raw,readonly=on,file=target/OVMF_VARS.fd \
		-drive format=raw,file=fat:rw:target/$(TARGET) \
		-net none
