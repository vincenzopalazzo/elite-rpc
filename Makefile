CC=cargo
FMT=fmt

ARGS=
TEST_LOG_LEVEL=

default: fmt
	$(CC) build --all-features

fmt:
	$(CC) fmt --all
	$(CC) clippy --workspace

check:
	$(CC) test --all -- --show-output

clean:
	$(CC) clean

install:
	$(CC) build --release
	$(CC) install --locked --path ./lampo-cli 
	$(CC) install --locked --path ./lampod-cli
	sudo cp target/release/liblampo.so /usr/local/lib

integration: default
	 TEST_LOG_LEVEL=$(TEST_LOG_LEVEL) $(CC) test -p tests $(ARGS)
