debug:
	cargo build

check: debug
	just tests/check
