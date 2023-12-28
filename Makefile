SHELL := bash
CC := $(shell which cargo)
GLSLC := $(shell which glslc)
TAR := $(shell which tar)
PIGZ := $(shell which pigz)
PWD := $(shell pwd)
PROJECT_NAME := $(shell pwd | sed "s#.*/##")
DOCKER_IMAGE_NAME := $(shell pwd | sed "s#.*/##" | tr [:upper:] [:lower:])
BIN := ${PROJECT_NAME}
SRC_DIR := src
SCRIPT_DIR := scripts
LIB_DIR := 
CARGO_TOML := Cargo.toml

all: build run

# Rust code
clean:
	(cd lve_rs && $(CC) clean)
	$(CC) clean

fmt:
	$(CC) fmt

build: fmt build-shaders
	(cd lve_rs && $(CC) build)
	$(CC) build

# Does not work inside docker containers!
build-frozen: fmt build-shaders
	(cd lve_rs && $(CC) build --frozen)
	$(CC) build --frozen

release: fmt build-shaders
	(cd lve_rs && $(CC) build --release)
	$(CC) build --release

run:
	./target/debug/${BIN}

build-shaders:
	$(GLSLC) shaders/simple_shader.frag -o shaders/simple_shader.frag.spv
	$(GLSLC) shaders/simple_shader.vert -o shaders/simple_shader.vert.spv
	$(GLSLC) shaders/point_light.frag -o shaders/point_light.frag.spv
	$(GLSLC) shaders/point_light.vert -o shaders/point_light.vert.spv

build-linux-image:
	cp Cargo.toml docker
	docker build . -t ${PROJECT_NAME}/linux -f docker/Dockerfile.linux
	rm docker/Cargo.toml

build-gdown-image:
	docker build . -t ${PROJECT_NAME}/gdown -f docker/Dockerfile.gdown

rebuild-linux-image:
	cp Cargo.toml docker
	docker build . -t ${DOCKER_IMAGE_NAME}/linux -f docker/Dockerfile.linux --no-cache
	rm docker/Cargo.toml

docker-build:
	docker run --rm -it -v $(shell pwd):/app ${DOCKER_IMAGE_NAME}/linux bash \
		-c "make build"

docker-release:
	docker run --rm -it -v $(shell pwd):/app ${DOCKER_IMAGE_NAME}/linux bash \
		-c "make release"

grab-models:
	docker run --rm -it -v $(shell pwd):/app ${DOCKER_IMAGE_NAME}/gdown sh \
		-c "source ${SCRIPT_DIR}/grab_models.sh && source ${SCRIPT_DIR}/grab-pastebin.sh quad"

docker-debug:
	docker run --rm -it -v $(shell pwd):/app ${DOCKER_IMAGE_NAME}/linux bash

docker-debug-gdown:
	docker run --rm -it -v $(shell pwd):/app ${DOCKER_IMAGE_NAME}/gdown sh

docker-run: docker-build run

docker-run-release: docker-release
	./target/release/${BIN}

compress: clean
	@cd ../ && ( \
		[ -f ${PIGZ} ] \
			&& $(TAR) --use-compress-program="pigz --best --recursive | pv" -cvf ${PROJECT_NAME}.tar.gz ${PROJECT_NAME} \
			|| $(TAR) czvf vulkan-tutorial.tar.gz ${PROJECT_NAME} \
	) 
