#include <iostream>

extern "C" void set_num_threads(int32_t nthreads);

extern "C" int32_t dinput(int32_t input);
extern "C" void print_hello_from_rust();
extern "C" void run_tmo(unsigned int width, unsigned int height, unsigned char *y, unsigned char *u, unsigned char *v, float* lum);
