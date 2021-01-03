CARGO = cargo +stable

all: client_test server unit_test release examples tests

server:
	$(CARGO) build

client_test:
	$(CARGO) build --features="client"

unit_test:
	$(CARGO) build --features="test"
	
release:
	$(CARGO) build --release


#.PHONY: tests
#tests:
#	$(CARGO) test --test client

.PHONY: examples
examples:
	$(CARGO) build --example b64
	$(CARGO) build --example simple
	$(CARGO) build --example snappy


clean:
	$(CARGO) clean



