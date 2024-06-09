run:
	RUST_BACKTRACE=1 RUST_LOG=error cargo run

clippy:
	cargo clippy

memory-profile:
	if [[ ! -d .venv ]]; then python -m venv .venv; ./.venv/bin/pip install -r requirements.txt; fi
	cargo build --release
	./.venv/bin/mprof run ./target/release/swift-tool-box -i static/2.xcactivitylog -o result.json
	./.venv/bin/mprof plot -o mprof.png
	xdg-open mprof.png

clear-memory-profile:
	rm mprof*.dat
	rm mprof*.png
