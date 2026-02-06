/**
 * C++ Header-Only Wrapper for plotffi
 *
 * This header provides a convenient C++ interface to the plotffi library.
 * Include this file and link against the plotffi shared library.
 */

#ifndef PLOTFFI_HPP
#define PLOTFFI_HPP

#include <stdexcept>
#include <string>
#include <vector>

// Include the C header (already has extern "C" guards)
#include "plotffi.h"

namespace plotffi {

/**
 * Exception thrown when plot rendering fails.
 */
class PlotError : public std::runtime_error {
public:
    explicit PlotError(const std::string& message)
        : std::runtime_error(message) {}
};

/**
 * Builder-style options for scatter plots.
 */
struct ScatterOptions {
    uint32_t width = 800;
    uint32_t height = 600;
    uint32_t markerRadius = 5;
    bool autoRange = true;
    double xMin = 0.0;
    double xMax = 1.0;
    double yMin = 0.0;
    double yMax = 1.0;

    ScatterOptions& setSize(uint32_t w, uint32_t h) {
        width = w;
        height = h;
        return *this;
    }

    ScatterOptions& setMarkerRadius(uint32_t radius) {
        markerRadius = radius;
        return *this;
    }

    ScatterOptions& setAutoRange(bool enabled) {
        autoRange = enabled;
        return *this;
    }

    ScatterOptions& setXRange(double min, double max) {
        xMin = min;
        xMax = max;
        autoRange = false;
        return *this;
    }

    ScatterOptions& setYRange(double min, double max) {
        yMin = min;
        yMax = max;
        autoRange = false;
        return *this;
    }

    /**
     * Converts to the C API struct.
     */
    PlotOptions toCOptions() const {
        PlotOptions opt;
        opt.width = width;
        opt.height = height;
        opt.marker_radius = markerRadius;
        opt.auto_range = autoRange ? 1 : 0;
        opt.x_min = xMin;
        opt.x_max = xMax;
        opt.y_min = yMin;
        opt.y_max = yMax;
        return opt;
    }
};

/**
 * Renders a scatter plot to a PNG file.
 *
 * @param path Output file path (PNG)
 * @param xs X coordinates
 * @param ys Y coordinates
 * @param options Plot configuration options
 * @throws PlotError if rendering fails
 */
inline void scatterPng(
    const std::string& path,
    const std::vector<double>& xs,
    const std::vector<double>& ys,
    const ScatterOptions& options = ScatterOptions())
{
    if (xs.size() != ys.size()) {
        throw PlotError("X and Y coordinate vectors must have the same size");
    }

    if (xs.empty()) {
        throw PlotError("Cannot create scatter plot with zero points");
    }

    PlotOptions cOpt = options.toCOptions();

    int result = plot_scatter_png(
        path.c_str(),
        xs.data(),
        ys.data(),
        xs.size(),
        cOpt
    );

    if (result != 0) {
        const char* errorMsg = plot_last_error_message();
        if (errorMsg != nullptr) {
            throw PlotError(std::string("Plot failed: ") + errorMsg);
        } else {
            throw PlotError("Plot failed with unknown error");
        }
    }
}

} // namespace plotffi

#endif // PLOTFFI_HPP
