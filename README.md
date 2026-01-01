# Edk2
cd ~/edk2/
cp /workspaces/x86_devcontainer/target.txt ~/edk2/Conf/target.txt
ln -s /workspaces/x86_devcontainer/mikanos/MikanLoaderPkg ./
source edksetup.sh 
build

# Kernel
cp target/x86_64-unknown-none-elf/debug/kernel kernel.elf
~/osbook/devenv/run_qemu.sh ~/edk2/Build/MikanLoaderX64/DEBUG_CLANG38/X64/Loader.efi /workspaces/x86_devcontainer/mikanos/kernel/kernel.elf