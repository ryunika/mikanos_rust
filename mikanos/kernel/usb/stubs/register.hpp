#pragma once

#include <cstdint>
#include <cstddef>

// Simple register wrapper for registers without named bit fields.
template <typename T>
union DefaultBitmap {
    T data[1];
    T value;

    operator T() const { return value; }
};

// Array wrapper for a contiguous array of register sets at a given MMIO address.
template <typename T>
struct ArrayWrapper {
    T* base_;
    size_t size_;

    ArrayWrapper(uintptr_t base, size_t size)
        : base_(reinterpret_cast<T*>(base)), size_(size) {}

    T& operator[](size_t i) { return base_[i]; }
    const T& operator[](size_t i) const { return base_[i]; }
    size_t Size() const { return size_; }
};

template <typename T>
struct MemMapRegister {
    volatile T data_;

    T Read() const {
        T tmp;
        if constexpr (sizeof(T) >= sizeof(uint32_t) &&
                      sizeof(T) % sizeof(uint32_t) == 0) {
            const volatile uint32_t* src =
                reinterpret_cast<const volatile uint32_t*>(&data_);
            uint32_t* dst = reinterpret_cast<uint32_t*>(&tmp);
            for (size_t i = 0; i < sizeof(T) / sizeof(uint32_t); ++i) {
                dst[i] = src[i];
            }
        } else if constexpr (sizeof(T) == 2) {
            *reinterpret_cast<uint16_t*>(&tmp) =
                *reinterpret_cast<const volatile uint16_t*>(&data_);
        } else {
            *reinterpret_cast<uint8_t*>(&tmp) =
                *reinterpret_cast<const volatile uint8_t*>(&data_);
        }
        return tmp;
    }

    void Write(const T& value) {
        if constexpr (sizeof(T) >= sizeof(uint32_t) &&
                      sizeof(T) % sizeof(uint32_t) == 0) {
            volatile uint32_t* dst =
                reinterpret_cast<volatile uint32_t*>(&data_);
            const uint32_t* src = reinterpret_cast<const uint32_t*>(&value);
            for (size_t i = 0; i < sizeof(T) / sizeof(uint32_t); ++i) {
                dst[i] = src[i];
            }
        } else if constexpr (sizeof(T) == 2) {
            *reinterpret_cast<volatile uint16_t*>(&data_) =
                *reinterpret_cast<const uint16_t*>(&value);
        } else {
            *reinterpret_cast<volatile uint8_t*>(&data_) =
                *reinterpret_cast<const uint8_t*>(&value);
        }
    }
};
