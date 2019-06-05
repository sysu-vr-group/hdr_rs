#include <iostream>
//-I/opt/X11/include -L /opt/X11/lib -lX11 -L ../target/release/ -lhdr_rs
#include "CImg.h"
#include <vector>
#include "hdr_rs.hpp"
#include <ctime>

using namespace cimg_library;
using namespace std;

int main() {
    CImg<float> src("../test.jpg");
    src.save("../origin.bmp");
    src.RGBtoYUV();
    vector<unsigned char> y,u,v;
    cimg_forXY(src, x, y_) {
        y.push_back(src(x, y_, 0) * 255);
        u.push_back(src(x, y_, 1) * 255);
        v.push_back(src(x, y_, 2) * 255);
    }
    clock_t start = clock();
    run_tmo(src.width(), src.height(), y.data(), u.data(), v.data());
    cout << "TMO Time: " << double(clock() - start) / CLOCKS_PER_SEC << endl;
    cimg_forXY(src, x, y_) {
        src(x, y_, 0) = float(y.at(y_*src.width()+x)) / 255;
    }
    src.YUVtoRGB();
    src.save("../out.bmp");
}

