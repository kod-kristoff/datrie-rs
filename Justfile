debug:
	cargo build

headers:
	@mkdir -pv dist/include/datrie
	@cp -v src/clib/include/datrie/* dist/include/datrie

dist: headers

check: debug dist
	just tests/check
