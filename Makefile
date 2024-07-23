.DEFAULT_GOAL := all

# ------------------------
#  Variables
# ------------------------
COMMIT_SHA:=$(shell git show --format='%H' --no-patch)
ETL_USER_NAME?=$(shell whoami | sed 's/[^0-9a-zA-Z-]//g')
TIMESTAMP:=$(shell date +%Y%m%d-%H%M%S)

# ------------------------
#  Commands
# ------------------------
.PHONY: all
all: build 

.PHONY: build_zc
build_zc: 
	cargo clean && cargo build -j 4 --release && sudo mv target/release/zc2 /usr/local/bin/zc

.PHONY: build
build: build_zc 
	docker compose down
	docker compose build
	docker compose up -d
	
.PHONY: run
run: build
	./zc

.PHONY: release
release: 
	./zc2
	
.PHONY: debug
debug: build
	RUST_BACKTRACE=full  ./zc
	