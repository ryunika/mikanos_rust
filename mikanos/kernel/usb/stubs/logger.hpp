#pragma once

#include <cstdint>
#include <cstring>
#include <array>
#include <algorithm>

enum LogLevel { kError = 0, kWarn, kInfo, kDebug };

inline void Log(int, const char*, ...) {}
