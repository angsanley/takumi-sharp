use std::{ffi::CStr, sync::LazyLock};

use takumi::{layout::node::NodeKind, rendering::RenderOptionsBuilder, GlobalContext};

static mut GLOBAL_CONTEXT: LazyLock<GlobalContext> = LazyLock::new(GlobalContext::default);
static mut GLOBAL_LAST_ERROR: Option<std::ffi::CString> = None;

unsafe fn set_last_error(err: impl std::fmt::Display) {
    let s = err.to_string();
    GLOBAL_LAST_ERROR = Some(std::ffi::CString::new(s).unwrap_or_else(|_| std::ffi::CString::new("Error creating error string").unwrap()));
}

/// The viewport for the image renderer.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    /// The width of the viewport in pixels, or -1 for none/auto.
    pub width: i32,
    /// The height of the viewport in pixels, or -1 for none/auto.
    pub height: i32,
    /// The font size in pixels, used for em and rem units.
    pub font_size: f32,
    /// The device pixel ratio
    pub device_pixel_ratio: f32,
}

impl From<Viewport> for takumi::layout::Viewport {
    fn from(value: Viewport) -> Self {
        Self {
            width: if value.width < 0 {
                None
            } else {
                Some(value.width as u32)
            },
            height: if value.height < 0 {
                None
            } else {
                Some(value.height as u32)
            },
            font_size: value.font_size,
            device_pixel_ratio: value.device_pixel_ratio,
        }
    }
}

#[repr(u8)]
pub enum ImageFormat {
    /// An Image in PNG Format
    Png,

    /// An Image in JPEG Format
    Jpeg,

    /// An Image in GIF Format
    Gif,

    /// An Image in WEBP Format
    WebP,

    /// An Image in general PNM Format
    Pnm,

    /// An Image in TIFF Format
    Tiff,

    /// An Image in TGA Format
    Tga,

    /// An Image in DDS Format
    Dds,

    /// An Image in BMP Format
    Bmp,

    /// An Image in ICO Format
    Ico,

    /// An Image in Radiance HDR Format
    Hdr,

    /// An Image in OpenEXR Format
    OpenExr,

    /// An Image in farbfeld Format
    Farbfeld,

    /// An Image in AVIF Format
    Avif,

    /// An Image in QOI Format
    Qoi,
}

impl From<ImageFormat> for takumi::image::ImageFormat {
    fn from(value: ImageFormat) -> Self {
        match value {
            ImageFormat::Png => Self::Png,
            ImageFormat::Jpeg => Self::Jpeg,
            ImageFormat::Gif => Self::Gif,
            ImageFormat::WebP => Self::WebP,
            ImageFormat::Pnm => Self::Pnm,
            ImageFormat::Tiff => Self::Tiff,
            ImageFormat::Tga => Self::Tga,
            ImageFormat::Dds => Self::Dds,
            ImageFormat::Bmp => Self::Bmp,
            ImageFormat::Ico => Self::Ico,
            ImageFormat::Hdr => Self::Hdr,
            ImageFormat::OpenExr => Self::OpenExr,
            ImageFormat::Farbfeld => Self::Farbfeld,
            ImageFormat::Avif => Self::Avif,
            ImageFormat::Qoi => Self::Qoi,
        }
    }
}

/// Loads and stores font data into the global font context.
///
/// # Safety
///
/// - `data` must be a valid pointer to a byte array of at least `len` bytes.
/// - The data must remain valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn global_font_context_load_and_store(data: *const u8, len: usize) -> bool {
    let data_arr = unsafe { std::slice::from_raw_parts(data, len) };
    if let Err(e) = unsafe { &mut *std::ptr::addr_of_mut!(GLOBAL_CONTEXT) }
        .font_context
        .load_and_store(data_arr, None, None)
    {
        unsafe { set_last_error(e) };
        return false;
    }
    true
}

/// Calculates the required buffer size for rendering a node to an image.
///
/// # Safety
///
/// - `node_str` must be a valid null-terminated C string pointer.
#[no_mangle]
pub unsafe extern "C" fn render_calculate_buffer_size_with_format(
    node_str: *const std::ffi::c_char,
    viewport: Viewport,
    format: ImageFormat,
) -> u64 {
    if node_str.is_null() {
        unsafe { set_last_error("node_str is null") };
        return 0;
    }

    let node_str = match CStr::from_ptr(node_str).to_str() {
        Ok(s) => s,
        Err(e) => {
            unsafe { set_last_error(e) };
            return 0;
        }
    };

    let node: NodeKind = match serde_json::from_str(node_str) {
        Ok(n) => n,
        Err(e) => {
            unsafe { set_last_error(e) };
            return 0;
        }
    };

    let opt = match RenderOptionsBuilder::default()
        .viewport(viewport.into())
        .node(node)
        .global(unsafe { &*std::ptr::addr_of!(GLOBAL_CONTEXT) })
        .build()
    {
        Ok(o) => o,
        Err(e) => {
            unsafe { set_last_error(e) };
            return 0;
        }
    };

    let img_format: takumi::image::ImageFormat = format.into();

    let image = match takumi::rendering::render(opt) {
        Ok(img) => img,
        Err(e) => {
            unsafe { set_last_error(e) };
            return 0;
        }
    };

    let mut cursor = std::io::Cursor::new(Vec::new());
    if let Err(e) = image.write_to(&mut cursor, img_format) {
        unsafe { set_last_error(e) };
        return 0;
    }

    cursor.into_inner().len() as u64
}

#[no_mangle]
pub unsafe extern "C" fn get_last_error() -> *const std::ffi::c_char {
    match unsafe { &*std::ptr::addr_of!(GLOBAL_LAST_ERROR) } {
        Some(s) => s.as_ptr(),
        None => std::ptr::null(),
    }
}

/// Renders a node to an image and writes it to a buffer.
///
/// # Safety
///
/// - `node_str` must be a valid null-terminated C string pointer.
/// - `out_buffer` must be a valid pointer to a buffer of at least `buffer_len` bytes.
/// - The buffer must remain valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn render_to_buffer_with_format(
    node_str: *const std::ffi::c_char,
    viewport: Viewport,
    format: ImageFormat,
    out_buffer: *mut u8,
    buffer_len: u64,
) -> bool {
    if node_str.is_null() || out_buffer.is_null() || buffer_len == 0 {
         unsafe { set_last_error("Invalid arguments: null pointer or zero buffer length") };
        return false;
    }

    let node_str = match CStr::from_ptr(node_str).to_str() {
        Ok(s) => s,
        Err(e) => {
            unsafe { set_last_error(e) };
            return false;
        }
    };

    let node: NodeKind = match serde_json::from_str(node_str) {
        Ok(n) => n,
        Err(e) => {
            unsafe { set_last_error(e) };
            return false;
        }
    };

    let opt = match RenderOptionsBuilder::default()
        .viewport(viewport.into())
        .node(node)
        .global(unsafe { &*std::ptr::addr_of!(GLOBAL_CONTEXT) })
        .build()
    {
        Ok(o) => o,
        Err(e) => {
            unsafe { set_last_error(e) };
            return false;
        }
    };

    let img_format: takumi::image::ImageFormat = format.into();

    let image = match takumi::rendering::render(opt) {
        Ok(img) => img,
        Err(e) => {
             unsafe { set_last_error(e) };
            return false;
        }
    };

    let mut cursor = std::io::Cursor::new(Vec::new());
    if let Err(e) = image.write_to(&mut cursor, img_format) {
         unsafe { set_last_error(e) };
        return false;
    }

    let bytes = cursor.into_inner();
    if bytes.len() > buffer_len as usize {
        unsafe { set_last_error("Buffer too small") };
        return false;
    }

    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buffer, bytes.len());
    true
}
