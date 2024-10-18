build:
	RUSTFLAGS="-C target-feature=+zba,+zbb,+zbc,+zbs --cfg debug_assertions" TARGET_CC="" cargo build --release --target riscv64imac-unknown-none-elf
run: build
	ckb-debugger --bin ./target/riscv64imac-unknown-none-elf/release/ckb-voting-test --tx-file ./mock_tx.json --cell-index 0 --cell-type input --script-group-type lock --max-cycles 3500000000
dump.txt: sign.bin
	hexdump -v -e '/1 "%02x"' ./sign.bin  > dump.txt
