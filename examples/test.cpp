#include <iostream>

extern "C" int32_t dinput(int32_t input);
extern "C" void print_hello_from_rust();

int main() {
    int input = 4;
    print_hello_from_rust();
    int output = dinput(input);
    printf("%d * 2 = %d\n", input, output);
    return 0;
}