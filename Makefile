BOARD := omnibusf4v3
GDB := gdb-multiarch

mass-erase := false

ifeq ($(shell uname),Linux)
	SUDO := sudo
endif

ifdef board
	BOARD = $(board)
endif

.PHONY: submodule
submodule:
	@git submodule update --init --recursive

.PHONY: $(BOARD)
$(BOARD): submodule
	@(cd boards/$(BOARD) && cargo build --release --target-dir ../../target)

define TARGET
	$(shell find target -name $(BOARD) -print -quit)
endef

define TEXT_ADDRESS
	$(shell arm-none-eabi-readelf -S $(TARGET) | grep .text | awk '{print "0x"$$5}')
endef

$(BOARD).dfu: $(BOARD)
	arm-none-eabi-objcopy -O binary -j .vtable $(TARGET) firmware0.bin
	arm-none-eabi-objcopy -O binary -R .vtable $(TARGET) firmware1.bin
	scripts/dfuse-pack.py -b 0x08000000:firmware0.bin -b $(TEXT_ADDRESS):firmware1.bin $(BOARD).dfu
	rm -f firmware0.bin firmware1.bin

.PHONY: dfu
dfu: $(BOARD).dfu

.PHONY: test
test:
	@cargo test

.PHONY: clean
clean:
	(cd boards/$(BOARD); cargo clean --target-dir ../../target)
	git submodule foreach git clean -dfX

.PHONY: dfu-upload
dfu-upload: dfu
ifeq ($(mass-erase),true)
	$(SUDO) dfu-util -d 0483:df11 -a 0 -s :mass-erase:force:leave -D $(BOARD).dfu
else
	$(SUDO) dfu-util -d 0483:df11 -a 0 -s :leave -D $(BOARD).dfu
endif

.PHONY: gdb
gdb:
	$(GDB) $(TARGET)

.PHONY: bloat
bloat:
	@(cd boards/$(BOARD); cargo bloat --release -n 10)

.DEFAULT_GOAL := $(BOARD)
