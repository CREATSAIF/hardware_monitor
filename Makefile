.PHONY: all build clean docker docker-build docker-run test

VERSION := 0.1.0

all: build

build:
	./build.sh

docker-build:
	docker-compose build builder

docker-run:
	docker-compose up hardware-monitor

docker: docker-build docker-run

clean:
	cargo clean
	rm -rf release/
	docker-compose down -v

test:
	cargo test --all-features

release: clean build
	@echo "Creating release package..."
	cd release && \
	echo "Release files:" && \
	ls -l && \
	echo "SHA256 checksums:" && \
	cat SHA256SUMS 