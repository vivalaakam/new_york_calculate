test_core:
	cargo check
	cargo test
	cargo fmt --all -- --check

test_python:
	source ./venv/bin/activate;
	pip install -r ./python/requirements.txt;
	cd ./python && maturin build;
	cd ./python && pytest;

test: test_core test_python