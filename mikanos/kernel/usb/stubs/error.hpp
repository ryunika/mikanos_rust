#pragma once

#include <cstdint>
#include <array>

struct Error {
    enum Code {
        kSuccess = 0,
        kFull,
        kEmpty,
        kNoEnoughMemory,
        kFromOSLayer,
        kUnknownXHCISpeedID,
        kNoWaiter,
        kNoPCIMSI,
        kAlreadyAllocated,
        kInvalidDescriptor,
        kInvalidSlotID,
        kTransferRingNotSet,
        kAlreadyNormal,
        kNotImplemented,
        kInvalidPhase,
        kUnknownDevice,
        kNoCorrespondingSetupStage,
        kTransferFailed,
        kInvalidEndpointNumber,
        kInvalidDeviceId,
        kBufferTooSmall,
        kEndpointNotInCharge,
        kPortNotConnected,
        kHostControllerNotHalted,
        kLastOfCode,
    };

    Code code;
    const char* file;
    int line;

    Error(Code code_, const char* file_ = "", int line_ = 0)
        : code(code_), file(file_), line(line_) {}

    operator bool() const { return code != kSuccess; }
    Code Cause() const { return code; }
    const char* Name() const { return "Error"; }
    const char* File() const { return file; }
    int Line() const { return line; }
};

template <typename T>
struct WithError {
    T value;
    Error error;
};

#define MAKE_ERROR(code) Error{(code), __FILE__, __LINE__}
