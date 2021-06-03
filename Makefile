.DEFAULT_GOAL := release

release:
	@echo "Creating release for current platform"
	cargo build --release

debug:
	@echo "Creating debug build for current platform"
	cargo build
