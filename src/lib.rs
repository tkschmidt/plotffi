//! FFI library for rendering scatter plots to PNG using Plotters.
//!
//! This library provides a C-compatible API for creating scatter plots.

use once_cell::sync::OnceCell;
use plotters::prelude::*;
use plotters::style::register_font;
use plotters_bitmap::BitMapBackend;
use std::ffi::{CStr, CString, c_char, c_double};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

// Embed the font file at compile time
static FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");

// Global storage for the last error message
static LAST_ERROR: Mutex<Option<CString>> = Mutex::new(None);

// Font registration happens once per process
static FONT_REGISTERED: OnceCell<Result<(), String>> = OnceCell::new();

/// Options for configuring the scatter plot.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PlotOptions {
    /// Width of the output image in pixels
    pub width: u32,
    /// Height of the output image in pixels
    pub height: u32,
    /// Radius of scatter plot markers in pixels
    pub marker_radius: u32,
    /// If nonzero, automatically compute axis ranges from data (with 2% padding)
    pub auto_range: u8,
    /// Minimum X axis value (used when auto_range == 0)
    pub x_min: c_double,
    /// Maximum X axis value (used when auto_range == 0)
    pub x_max: c_double,
    /// Minimum Y axis value (used when auto_range == 0)
    pub y_min: c_double,
    /// Maximum Y axis value (used when auto_range == 0)
    pub y_max: c_double,
}

/// Stores an error message for later retrieval via plot_last_error_message().
fn set_error(msg: String) {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        // Convert to CString, replacing any interior NUL bytes
        let sanitized = msg.replace('\0', "\\0");
        *guard = CString::new(sanitized).ok();
    }
}

/// Clears the stored error message.
fn clear_error() {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = None;
    }
}

/// Ensures the bundled font is registered with Plotters.
fn ensure_font_registered() -> Result<(), String> {
    FONT_REGISTERED
        .get_or_init(|| {
            register_font("app-font", FontStyle::Normal, FONT_BYTES)
                .map_err(|_| "Failed to register bundled font: invalid font data".to_string())
        })
        .clone()
}

/// Internal implementation of scatter plot rendering.
///
/// This function is public for benchmarking purposes.
#[doc(hidden)]
pub fn plot_scatter_png_impl(path: &str, xs: &[f64], ys: &[f64], opt: PlotOptions) -> Result<(), String> {
    // Ensure font is registered
    ensure_font_registered()?;

    // Validate dimensions
    if opt.width == 0 || opt.height == 0 {
        return Err("Width and height must be greater than zero".to_string());
    }

    // Compute axis ranges
    let (x_min, x_max, y_min, y_max) = if opt.auto_range != 0 {
        // Auto-compute from data with 2% padding
        let x_data_min = xs.iter().copied().fold(f64::INFINITY, f64::min);
        let x_data_max = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let y_data_min = ys.iter().copied().fold(f64::INFINITY, f64::min);
        let y_data_max = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        let x_range = x_data_max - x_data_min;
        let y_range = y_data_max - y_data_min;

        // Handle case where all points have the same coordinate
        let x_padding = if x_range.abs() < f64::EPSILON {
            1.0
        } else {
            x_range * 0.02
        };
        let y_padding = if y_range.abs() < f64::EPSILON {
            1.0
        } else {
            y_range * 0.02
        };

        (
            x_data_min - x_padding,
            x_data_max + x_padding,
            y_data_min - y_padding,
            y_data_max + y_padding,
        )
    } else {
        // Use explicit ranges from options
        if opt.x_min >= opt.x_max {
            return Err(format!(
                "Invalid X range: x_min ({}) must be less than x_max ({})",
                opt.x_min, opt.x_max
            ));
        }
        if opt.y_min >= opt.y_max {
            return Err(format!(
                "Invalid Y range: y_min ({}) must be less than y_max ({})",
                opt.y_min, opt.y_max
            ));
        }
        (opt.x_min, opt.x_max, opt.y_min, opt.y_max)
    };

    // Create the bitmap backend
    let root = BitMapBackend::new(path, (opt.width, opt.height)).into_drawing_area();

    // Fill background white
    root.fill(&WHITE)
        .map_err(|e| format!("Failed to fill background: {}", e))?;

    // Build chart with label areas
    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|e| format!("Failed to build chart: {}", e))?;

    // Configure and draw mesh (ticks/grid) with bundled font
    chart
        .configure_mesh()
        .label_style(("app-font", 14).into_font())
        .axis_desc_style(("app-font", 16).into_font())
        .draw()
        .map_err(|e| format!("Failed to draw mesh: {}", e))?;

    // Draw scatter points as filled circles
    let marker_radius = opt.marker_radius as i32;
    chart
        .draw_series(
            xs.iter()
                .zip(ys.iter())
                .map(|(&x, &y)| Circle::new((x, y), marker_radius, BLUE.filled())),
        )
        .map_err(|e| format!("Failed to draw points: {}", e))?;

    // Finalize and write PNG
    root.present().map_err(|e| format!("Failed to write PNG: {}", e))?;

    Ok(())
}

/// Renders a scatter plot to a PNG file.
///
/// # Parameters
/// - `path`: NUL-terminated UTF-8 path to the output PNG file
/// - `xs`: Pointer to array of X coordinates
/// - `ys`: Pointer to array of Y coordinates
/// - `n`: Number of points (length of xs and ys arrays)
/// - `opt`: Plot configuration options
///
/// # Returns
/// - 0 on success
/// - 1 on failure (call `plot_last_error_message()` for details)
///
/// # Safety
/// - `path` must be a valid NUL-terminated UTF-8 string
/// - `xs` and `ys` must point to arrays of at least `n` elements
/// - `n` must be greater than 0
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plot_scatter_png(
    path: *const c_char,
    xs: *const c_double,
    ys: *const c_double,
    n: usize,
    opt: PlotOptions,
) -> i32 {
    // Clear any previous error
    clear_error();

    // Wrap everything in catch_unwind to prevent panics crossing FFI boundary
    let result = catch_unwind(AssertUnwindSafe(|| {
        // Validate path pointer
        if path.is_null() {
            return Err("Path pointer is NULL".to_string());
        }

        // Validate data pointers
        if xs.is_null() {
            return Err("X data pointer is NULL".to_string());
        }
        if ys.is_null() {
            return Err("Y data pointer is NULL".to_string());
        }

        // Validate count
        if n == 0 {
            return Err("Point count (n) must be greater than zero".to_string());
        }

        // Convert path to Rust string
        let path_cstr = unsafe { CStr::from_ptr(path) };
        let path_str = path_cstr.to_str().map_err(|_| "Path is not valid UTF-8".to_string())?;

        // Create slices from raw pointers
        let xs_slice = unsafe { std::slice::from_raw_parts(xs, n) };
        let ys_slice = unsafe { std::slice::from_raw_parts(ys, n) };

        // Call implementation
        plot_scatter_png_impl(path_str, xs_slice, ys_slice, opt)
    }));

    match result {
        Ok(Ok(())) => 0,
        Ok(Err(msg)) => {
            set_error(msg);
            1
        },
        Err(panic_info) => {
            let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                format!("Internal panic: {}", s)
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                format!("Internal panic: {}", s)
            } else {
                "Internal panic (unknown cause)".to_string()
            };
            set_error(msg);
            1
        },
    }
}

/// Returns the last error message, or NULL if no error has occurred.
///
/// The returned pointer is valid until the next call to `plot_scatter_png()`.
/// The string is NUL-terminated UTF-8.
///
/// # Safety
/// The returned pointer must not be freed by the caller.
#[unsafe(no_mangle)]
pub extern "C" fn plot_last_error_message() -> *const c_char {
    match LAST_ERROR.lock() {
        Ok(guard) => match &*guard {
            Some(cstring) => cstring.as_ptr(),
            None => std::ptr::null(),
        },
        Err(_) => std::ptr::null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::fs;

    #[test]
    fn test_basic_plot() {
        let path = CString::new("/tmp/test_scatter.png").unwrap();
        let xs: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ys: Vec<f64> = vec![1.0, 4.0, 2.0, 3.0, 5.0];
        let opt = PlotOptions {
            width: 800,
            height: 600,
            marker_radius: 5,
            auto_range: 1,
            x_min: 0.0,
            x_max: 0.0,
            y_min: 0.0,
            y_max: 0.0,
        };

        let result = unsafe { plot_scatter_png(path.as_ptr(), xs.as_ptr(), ys.as_ptr(), xs.len(), opt) };

        assert_eq!(result, 0, "Expected success");
        assert!(fs::metadata("/tmp/test_scatter.png").is_ok());
        fs::remove_file("/tmp/test_scatter.png").ok();
    }

    #[test]
    fn test_null_path() {
        let xs: Vec<f64> = vec![1.0, 2.0];
        let ys: Vec<f64> = vec![1.0, 2.0];
        let opt = PlotOptions {
            width: 800,
            height: 600,
            marker_radius: 5,
            auto_range: 1,
            x_min: 0.0,
            x_max: 0.0,
            y_min: 0.0,
            y_max: 0.0,
        };

        let result = unsafe { plot_scatter_png(std::ptr::null(), xs.as_ptr(), ys.as_ptr(), xs.len(), opt) };

        assert_eq!(result, 1, "Expected failure for NULL path");
        let err = plot_last_error_message();
        assert!(!err.is_null());
    }

    #[test]
    fn test_zero_count() {
        let path = CString::new("/tmp/test_zero.png").unwrap();
        let xs: Vec<f64> = vec![];
        let ys: Vec<f64> = vec![];
        let opt = PlotOptions {
            width: 800,
            height: 600,
            marker_radius: 5,
            auto_range: 1,
            x_min: 0.0,
            x_max: 0.0,
            y_min: 0.0,
            y_max: 0.0,
        };

        let result = unsafe { plot_scatter_png(path.as_ptr(), xs.as_ptr(), ys.as_ptr(), 0, opt) };

        assert_eq!(result, 1, "Expected failure for zero count");
    }

    #[test]
    fn test_explicit_range() {
        let path = CString::new("/tmp/test_explicit.png").unwrap();
        let xs: Vec<f64> = vec![1.0, 2.0, 3.0];
        let ys: Vec<f64> = vec![1.0, 2.0, 3.0];
        let opt = PlotOptions {
            width: 800,
            height: 600,
            marker_radius: 5,
            auto_range: 0,
            x_min: 0.0,
            x_max: 10.0,
            y_min: 0.0,
            y_max: 10.0,
        };

        let result = unsafe { plot_scatter_png(path.as_ptr(), xs.as_ptr(), ys.as_ptr(), xs.len(), opt) };

        assert_eq!(result, 0, "Expected success with explicit range");
        fs::remove_file("/tmp/test_explicit.png").ok();
    }
}
