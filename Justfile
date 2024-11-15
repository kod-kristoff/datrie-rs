debug:
	cargo build

headers:
	@mkdir -pv dist/include/datrie
	@cp -v crates/datrie-clib/include/datrie/* dist/include/datrie

dist: headers

rust_check:
	cargo test --all -- --show-output

check: debug dist rust_check
	just tests/check
