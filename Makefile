all:
	cargo build
	sudo ./deploy.sh
	curl -v localhost:8080
