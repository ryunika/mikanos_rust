KERNEL_DIR  := /workspace/mikanos/kernel
OSBOOK_DIR  := /home/vscode/osbook/devenv/x86_64-elf

CXX      := clang++
CXXFLAGS := -O2 -Wall -g \
             -mno-red-zone \
             -fno-exceptions \
             -fno-rtti \
             -std=c++17 \
             -nostdinc++ \
             -isystem $(OSBOOK_DIR)/include/c++/v1 \
             -I$(KERNEL_DIR)/usb/stubs \
             -I$(KERNEL_DIR)

SRCS := lib.cpp \
        driver/memory.cpp \
        driver/device.cpp \
        driver/xhci/xhci.cpp \
        driver/xhci/registers.cpp \
        driver/xhci/trb.cpp \
        driver/xhci/ring.cpp \
        driver/xhci/port.cpp \
        driver/xhci/device.cpp \
        driver/xhci/devmgr.cpp \
        driver/classdriver/base.cpp \
        driver/classdriver/hid.cpp \
        driver/classdriver/mouse.cpp \
        driver/classdriver/keyboard.cpp \
        driver/classdriver/cdc.cpp

OBJS := $(patsubst %.cpp,%.o,$(SRCS))

.PHONY: all clean

all: libusb.a

libusb.a: $(OBJS)
	ar crs $@ $^

%.o: %.cpp
	$(CXX) $(CXXFLAGS) -c $< -o $@

clean:
	rm -f $(OBJS) libusb.a
