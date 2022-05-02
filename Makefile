SHELL := /bin/bash

build_linux:
	CC_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-gcc \
	CXX_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-g++ \
	AR_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-ar \
	CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc \
	cargo build --target x86_64-unknown-linux-gnu

test:
	cargo test $(T) -- --nocapture --test-threads=1

.PHONY: test