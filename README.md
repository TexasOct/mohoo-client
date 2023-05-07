# mohoo-client

OpenWRT mohoo client program for ramips.

本项目设计是为配合NEXX WT3020路由器8M版本做出了一定优化的openwrt客户端。

使用到的核心cagro有
```
rocket api提供
rust-uci 提供abi级别的uci交互
wirguard-control 提供abi级别的wireguard交互
```

此为scumaker协会内部项目客户端，需配合特定 wireguard server 使用。


# build

run the container file environments and build in it 
```shell
cd mohoo-client
mkdir build
podman build -t mohoo-client .
podman run --rm -it -v $(pwd):/mnt:z -w /mnt localhost/mohoo-build:latest  bash # option z with SElinux
make all
```