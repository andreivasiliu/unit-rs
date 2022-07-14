all: debug

debug:
	cargo build
	sudo ./deploy.sh
	curl -v localhost:8080

release:
	cargo build --release
	sudo ./deploy.sh release
	curl -v localhost:8080

bench:
	wrk -c 32 -d 3 -t 8 http://localhost:8080
