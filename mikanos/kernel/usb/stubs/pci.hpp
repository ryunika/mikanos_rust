#pragma once

#include <cstdint>
#include "error.hpp"

namespace pci {
    struct ClassCode {
        uint8_t base, sub, interface;
        bool Match(uint8_t b, uint8_t s, uint8_t i) const {
            return base == b && sub == s && interface == i;
        }
    };

    struct Device {
        uint8_t bus, device, function, header_type;
        ClassCode class_code;
    };

    inline Device devices[32];
    inline int num_device = 0;

    inline uint16_t ReadVendorId(const Device&) { return 0; }
    inline WithError<uint64_t> ReadBar(Device&, uint8_t) {
        return {0, Error{Error::kSuccess}};
    }

    enum class MSITriggerMode { kEdge, kLevel };
    enum class MSIDeliveryMode { kFixed };

    inline void ConfigureMSIFixedDestination(
        Device&, uint8_t, MSITriggerMode, MSIDeliveryMode, uint8_t, uint8_t) {}

    inline uint32_t ReadConfReg(const Device&, uint8_t) { return 0; }
    inline void WriteConfReg(const Device&, uint8_t, uint32_t) {}
}
