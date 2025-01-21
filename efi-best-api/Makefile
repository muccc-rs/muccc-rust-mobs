target/x86_64-unknown-uefi/release/efi-best-api.efi:
	 cargo build --release --target x86_64-unknown-uefi

esp/efi/boot/bootx64.efi: target/x86_64-unknown-uefi/release/efi-best-api.efi
	mkdir -p esp/efi/boot
	cp $< $@
run: esp/efi/boot/bootx64.efi
	qemu-system-x86_64 -enable-kvm -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd -drive format=raw,file=fat:rw:esp -device virtio-rng-pci
.PHONY: target/x86_64-unknown-uefi/release/efi-best-api.efi
	
