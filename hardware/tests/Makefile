HARNESS_DIR := $(abspath ../../harness)
TEST_DIR := $(abspath .)
TARGET := mipsel-sony-psx
SERIAL_PORT=$(shell ls /dev/tty.usbserial-* 2>/dev/null | head -n 1)
bin_name=$(shell basename $(bin) .bin)

.PHONY: all build

all: build

build:
	@if [ ! -f $(bin) ]; then \
	  echo "Error: missing binary file in $(TEST_DIR)"; exit 1; \
	fi
	@echo "→ Building harness with stub: $(bin)"
	@TEST_BIN="$(bin)" \
	  cargo psx build \
	  	--manifest-path $(HARNESS_DIR)/Cargo.toml \
		--target-dir $(TEST_DIR)/harness
	
	# Extract basename of $(bin), replace .bin with .exe

	@mv $(TEST_DIR)/harness/$(TARGET)/release/harness.exe $(TEST_DIR)/$(bin_name).exe
	@rm -rf $(TEST_DIR)/harness
	
ship_it:
	echo ${SERIAL_PORT}

	@mono ~/projects/NOTPSXSerial/nops.exe /m /exe $(TEST_DIR)/$(bin) ${SERIAL_PORT}