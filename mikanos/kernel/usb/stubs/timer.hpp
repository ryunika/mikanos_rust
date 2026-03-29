#pragma once

#include <cstdint>

const int kTimerFreq = 100;

struct TimerManager {
    uint64_t CurrentTick() {
        static volatile uint64_t tick = 0;
        return tick++;
    }
};

inline TimerManager timer_manager_instance;
inline TimerManager* timer_manager = &timer_manager_instance;
