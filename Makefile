CARGO = cargo +stable

all: client_test server unit_test release

server:
	$(CARGO) build

client_test:
	$(CARGO) build --features="client"
	cp target/debug/yabts target/debug/client_test

unit_test:
	$(CARGO) build --features="test"
	
release:
	$(CARGO) build --release

clean:
	$(CARGO) clean



