#ifndef CONTAINER_STACK_HLSLI
#define CONTAINER_STACK_HLSLI

#include "../config.hlsli"

class Stack {
    uint stack[MAX_STACK_DEPTH];
    uint ptr;

    void push(uint value) {
        stack[ptr] = value;
        ptr++;
    }

    uint pop() {
        ptr--;
        uint value = stack[ptr];
        return value;
    }
};

Stack NewStack() {
    Stack stack;
    stack.ptr = 0;
    return stack;
}

#endif // CONTAINER_STACK_HLSLI
