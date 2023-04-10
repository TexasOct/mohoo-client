# Configuration
OPENWRT_VERSION := 22.03.3
OPENWRT_ARCH := ramips
OPENWRT_CHIP := mt7620
OPENWRT_HOST_ARCH := Linux-x86_64

OPENWRT_PROFILE := nexx_wt3020-8m
OPENWRT_PACKAGES := kmod-rt2800-lib kmod-rt2800-mmio kmod-rt2800-soc kmod-rt2x00-lib kmod-rt2x00-mmio
OPENWRT_PACKAGES := $(OPENWRT_PACKAGES) wireguard-tools coreutils-realpath coreutils-dirname
OPENWRT_FILES_DIR := openwrt_config
TOOLCHAIN_GCC_VERSION := gcc-11.2.0_musl
# Passing ${DIR} from command line to do out-of-tree build
BUILD_DIR := $(if $(DIR),$(DIR),".")
PROJECT_DIR := $(shell pwd)
########
# Global shortcuts
WGET := wget -q --show-progress
IMAGE_TOOLCHAIN := openwrt-imagebuilder-${OPENWRT_VERSION}-${OPENWRT_ARCH}-${OPENWRT_CHIP}.${OPENWRT_HOST_ARCH}
GCC_TOOLCHAIN := openwrt-sdk-${OPENWRT_VERSION}-${OPENWRT_ARCH}-${OPENWRT_CHIP}_${TOOLCHAIN_GCC_VERSION}.${OPENWRT_HOST_ARCH}
BUILD_TOOLCHAIN_NAME := toolchain-mipsel_24kc_${TOOLCHAIN_GCC_VERSION}
IMAGE_TOOLCHAIN_FILENAME := ${IMAGE_TOOLCHAIN}.tar.xz

########
# Rust Configuration
RUST_TARGET := mipsel-unknown-linux-musl
RUST_NAME := mohoo-cli
########

STAGING_DIR := ${PROJECT_DIR}/${GCC_TOOLCHAIN}/staging_dir
PATH := ${PATH}:${PROJECT_DIR}/${GCC_TOOLCHAIN}/staging_dir/toolchain-mipsel_24kc_${TOOLCHAIN_GCC_VERSION}/bin

${BUILD_DIR}/${IMAGE_TOOLCHAIN_FILENAME}:
	${WGET} "https://downloads.openwrt.org/releases/${OPENWRT_VERSION}/targets/${OPENWRT_ARCH}/${OPENWRT_CHIP}/${IMAGE_TOOLCHAIN_FILENAME}" -O ${BUILD_DIR}/${IMAGE_TOOLCHAIN_FILENAME}

${BUILD_DIR}/${IMAGE_TOOLCHAIN}: ${BUILD_DIR}/${IMAGE_TOOLCHAIN_FILENAME}
	tar -xf ${BUILD_DIR}/${IMAGE_TOOLCHAIN_FILENAME} -C ${BUILD_DIR}


build_cli:
# 	sudo dnf install gcc clang
	cargo build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release
	cp ${PROJECT_DIR}/target/${RUST_TARGET}/release/${RUST_NAME} ${PROJECT_DIR}/${OPENWRT_FILES_DIR}/etc/root/


image_builder: ${BUILD_DIR}/${IMAGE_TOOLCHAIN}


build_openwrt: image_builder build_cli
	make -C ${BUILD_DIR}/${IMAGE_TOOLCHAIN} \
		PROFILE="${OPENWRT_PROFILE}" \
		PACKAGES="${OPENWRT_PACKAGES}" \
		FILES="${PROJECT_DIR}/${OPENWRT_FILES_DIR}" \
		image


copy_result: build_openwrt
	cp ${BUILD_DIR}/${IMAGE_TOOLCHAIN}/bin/targets/${OPENWRT_ARCH}/${OPENWRT_CHIP}/openwrt-${OPENWRT_VERSION}-${OPENWRT_ARCH}-${OPENWRT_CHIP}-${OPENWRT_PROFILE}-squashfs-*.bin ${BUILD_DIR}


all: copy_result


clean:
	cargo clean
	make -C ${BUILD_DIR}/${IMAGE_TOOLCHAIN} clean
	rm -f "${BUILD_DIR}/openwrt-${OPENWRT_VERSION}-${OPENWRT_ARCH}-${OPENWRT_CHIP}-${OPENWRT_PROFILE}-squashfs-"*".bin"


clean_all: clean
	rm -f "${BUILD_DIR}/${IMAGE_TOOLCHAIN_FILENAME}"
	rm -rf "${BUILD_DIR}/${IMAGE_TOOLCHAIN}"


.PHONY: build_cli build_openwrt image_builder clean clean_all copy_result
