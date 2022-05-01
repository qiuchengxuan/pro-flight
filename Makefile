BOARD := simulator
GDB := gdb-multiarch

DFU_FLAGS = :leave
mass-erase := false
ifeq ($(mass-erase),true)
	DFU_FLAGS = :mass-erase:force:leave
endif

CARGO_FEATURES =
ifeq ($(debug),true)
	CARGO_FEATURES = debug
endif
ifeq ($(semihosting),true)
	CARGO_FEATURES += cortex-m-semihosting
endif

ifeq ($(shell uname),Linux)
	SUDO := sudo
endif

ifdef board
	BOARD = $(board)
endif

.PHONY: submodule
submodule:
	@git submodule update --init --recursive

.PHONY: test
test: submodule
	@cargo test --all

.PHONY: $(BOARD)
$(BOARD): submodule
	(cd boards/$(BOARD) && cargo build --release --features "$(CARGO_FEATURES)" --target-dir ../../target)

SIMULATOR := $(PWD)/target/release/simulator
SIMULATOR_CONFIG := $(PWD)/boards/simulator/rascal.yaml
.PHONY: jsbsim
jsbsim: $(BOARD) simulator
	(cd tests/jsbsim; ./run.py --simulator $(SIMULATOR) --simulator-config $(SIMULATOR_CONFIG))

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

.PHONY: dfu-program
dfu-program: $(BOARD).dfu
	$(SUDO) dfu-util -d 0483:df11 -a 0 -s $(DFU_FLAGS) -D $(BOARD).dfu

.PHONY: gdb
gdb:
	$(GDB) $(TARGET)

.PHONY: bloat
bloat:
	@(cd boards/$(BOARD); cargo bloat --release -n 10)

.PHONY: clean
clean:
	(cd boards/$(BOARD); cargo clean --target-dir ../../target)
	git submodule foreach git clean -dfX

DEFAULT_RULE := $(BOARD).dfu
ifeq ($(BOARD),simulator)
	DEFAULT_RULE = $(BOARD)
endif

.PHONY: default
default: $(DEFAULT_RULE)

.DEFAULT_GOAL := default
