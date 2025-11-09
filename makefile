BINARY_NAMES := $(strip $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[].targets[] | select(.kind[] == "bin") | .name'))
TARGET_DIR := $(HOME)/.config/eww/scripts

update: build copy

build:
	cargo build --release

copy: build
	@mkdir -p "$(TARGET_DIR)"

	@for BINARY_NAME in $(BINARY_NAMES); do \
		BINARY="target/release/$$BINARY_NAME"; \
		if [ -f "$$BINARY" ]; then \
			echo "copying $$BINARY -> $(TARGET_DIR)"; \
			cp "$$BINARY" "$(TARGET_DIR)/"; \
		else \
			echo "warning: file $$BINARY_NAME does not exist"; \
		fi; \
	done

clean:
	cargo clean

.PHONY: update build copy clean
