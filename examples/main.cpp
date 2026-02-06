/**
 * Example usage of plotffi library from C++.
 *
 * Demonstrates:
 * - Basic scatter plot with auto-ranging
 * - Scatter plot with explicit axis ranges
 * - Error handling
 */

#include <cmath>
#include <iostream>
#include <vector>

#include "plotffi.hpp"

int main() {
    try {
        // Example 1: Simple scatter plot with auto-ranging
        {
            std::vector<double> xs = {1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0};
            std::vector<double> ys = {2.1, 3.9, 6.2, 7.8, 10.1, 12.0, 14.1, 15.9, 18.2, 19.8};

            plotffi::scatterPng("scatter_auto.png", xs, ys);
            std::cout << "Created scatter_auto.png (auto-range)\n";
        }

        // Example 2: Scatter plot with custom options
        {
            std::vector<double> xs, ys;

            // Generate some sine wave data with noise
            for (int i = 0; i < 50; ++i) {
                double x = i * 0.2;
                double y = std::sin(x) + (std::rand() % 100 - 50) / 500.0;
                xs.push_back(x);
                ys.push_back(y);
            }

            plotffi::ScatterOptions opts;
            opts.setSize(1024, 768)
                .setMarkerRadius(4)
                .setAutoRange(true);

            plotffi::scatterPng("scatter_sine.png", xs, ys, opts);
            std::cout << "Created scatter_sine.png (sine wave)\n";
        }

        // Example 3: Scatter plot with explicit axis ranges
        {
            std::vector<double> xs = {0.1, 0.2, 0.3, 0.4, 0.5};
            std::vector<double> ys = {0.1, 0.4, 0.9, 1.6, 2.5};

            plotffi::ScatterOptions opts;
            opts.setSize(640, 480)
                .setMarkerRadius(8)
                .setXRange(0.0, 1.0)
                .setYRange(0.0, 3.0);

            plotffi::scatterPng("scatter_explicit.png", xs, ys, opts);
            std::cout << "Created scatter_explicit.png (explicit range)\n";
        }

        // Example 4: Using the C API directly
        {
            std::vector<double> xs = {1.0, 2.0, 3.0};
            std::vector<double> ys = {1.0, 2.0, 3.0};

            PlotOptions opt;
            opt.width = 400;
            opt.height = 300;
            opt.marker_radius = 6;
            opt.auto_range = 1;
            opt.x_min = 0.0;
            opt.x_max = 0.0;
            opt.y_min = 0.0;
            opt.y_max = 0.0;

            int result = plot_scatter_png(
                "scatter_c_api.png",
                xs.data(),
                ys.data(),
                xs.size(),
                opt
            );

            if (result == 0) {
                std::cout << "Created scatter_c_api.png (C API)\n";
            } else {
                const char* err = plot_last_error_message();
                std::cerr << "C API error: " << (err ? err : "unknown") << "\n";
            }
        }

        std::cout << "\nAll plots created successfully!\n";
        return 0;

    } catch (const plotffi::PlotError& e) {
        std::cerr << "Plot error: " << e.what() << "\n";
        return 1;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }
}
