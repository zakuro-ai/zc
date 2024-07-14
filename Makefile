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

.PHONY: build
build: 
	cargo build 

.PHONY: run
run: build
	./zc

.PHONY: release
release: 
	./zc2
	
.PHONY: debug
debug: build
	RUST_BACKTRACE=full  ./zc
	