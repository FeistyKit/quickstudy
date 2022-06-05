quickstudy: $(shell find src -type f)
	cargo build --release && cp target/release/quickstudy .
