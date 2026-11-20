MAKEFLAGS += -rR
.SUFFIXES:

LIMINE_VERSION := 12.10.0
LIMINE_URL := https://github.com/Limine-Bootloader/Limine/releases/download/v$(LIMINE_VERSION)/limine-binary.tar.xz
LIMINE_DIR := limine-binary

IMAGE := kaguya.iso
ISO_ROOT := iso_root
KERNEL := target/x86_64-unknown-none/debug/kaguya

.PHONY: all run clean distclean kernel limine

all: $(IMAGE)

kernel:
	cargo build

limine:
	@if [ ! -x $(LIMINE_DIR)/limine ]; then \
		if [ ! -f $(LIMINE_DIR)/Makefile ]; then \
			curl -fL -o limine.tar.xz "$(LIMINE_URL)"; \
			tar -xf limine.tar.xz; \
			rm limine.tar.xz; \
		fi; \
		$(MAKE) -C $(LIMINE_DIR); \
	fi

$(IMAGE): kernel limine limine.conf
	rm -rf $(ISO_ROOT)
	mkdir -p $(ISO_ROOT)/boot/limine
	mkdir -p $(ISO_ROOT)/EFI/BOOT

	cp -v $(KERNEL) $(ISO_ROOT)/boot/kaguya
	cp -v limine.conf $(ISO_ROOT)/boot/limine/limine.conf

	cp -v $(LIMINE_DIR)/limine-bios.sys \
	      $(LIMINE_DIR)/limine-bios-cd.bin \
	      $(LIMINE_DIR)/limine-uefi-cd.bin \
	      $(ISO_ROOT)/boot/limine/

	cp -v $(LIMINE_DIR)/BOOTX64.EFI \
	      $(LIMINE_DIR)/BOOTIA32.EFI \
	      $(ISO_ROOT)/EFI/BOOT/

	xorriso -as mkisofs \
		-b boot/limine/limine-bios-cd.bin \
		-no-emul-boot \
		-boot-load-size 4 \
		-boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part \
		--efi-boot-image \
		--protective-msdos-label \
		$(ISO_ROOT) -o $(IMAGE)

	./$(LIMINE_DIR)/limine bios-install $(IMAGE)
	rm -rf $(ISO_ROOT)

run: $(IMAGE)
	qemu-system-x86_64 -M q35 -m 2G -cdrom $(IMAGE) -boot d -serial stdio

clean:
	rm -rf $(ISO_ROOT) $(IMAGE)

distclean: clean
	rm -rf $(LIMINE_DIR)
	rm -rf target
