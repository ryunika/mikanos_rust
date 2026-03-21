#!/bin/sh

cd ~/edk2/
cp /workspace/target.txt ~/edk2/Conf/target.txt
[ -L MikanLoaderPkg ] || ln -s /workspace/mikanos/MikanLoaderPkg/ ./
