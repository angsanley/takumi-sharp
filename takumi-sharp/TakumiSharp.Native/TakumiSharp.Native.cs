using System.Runtime.InteropServices;
using System.Text;

namespace TakumiSharp.Native;

/// <summary>
/// Low-level wrapper around the native Takumi library.
/// </summary>
public static unsafe class TakumiSharpNative
{
    /// <summary>
    /// Loads a font from the specified byte data and stores it in the global font context.
    /// </summary>
    /// <param name="fontData">The font file bytes</param>
    /// <returns>True if successful, false otherwise</returns>
    public static bool LoadFont(ReadOnlySpan<byte> fontData)
    {
        fixed (byte* dataPtr = fontData)
        {
            return Generated.global_font_context_load_and_store(dataPtr, (nuint)fontData.Length);
        }
    }

    /// <summary>
    /// Calculates the required buffer size for rendering a node to the specified image format.
    /// </summary>
    /// <param name="nodeJson">JSON string representing the node to render</param>
    /// <param name="viewport">Viewport settings for rendering</param>
    /// <param name="format">The image format to encode to</param>
    /// <returns>The required buffer size in bytes, or 0 if an error occurred</returns>
    public static ulong CalculateBufferSize(string nodeJson, Viewport viewport, ImageFormat format)
    {
        byte[] nodeBytes = Encoding.UTF8.GetBytes(nodeJson + '\0');

        fixed (byte* nodePtr = nodeBytes)
        {
            return Generated.render_calculate_buffer_size_with_format(nodePtr, viewport, format);
        }
    }

    /// <summary>
    /// Renders a node to a buffer in the specified image format.
    /// </summary>
    /// <param name="nodeJson">JSON string representing the node to render</param>
    /// <param name="viewport">Viewport settings for rendering</param>
    /// <param name="format">The image format to encode to</param>
    /// <param name="buffer">The buffer to write the encoded image data to</param>
    /// <returns>True if successful, false otherwise</returns>
    public static bool RenderToBuffer(string nodeJson, Viewport viewport, ImageFormat format, Span<byte> buffer)
    {
        byte[] nodeBytes = Encoding.UTF8.GetBytes(nodeJson + '\0');

        fixed (byte* nodePtr = nodeBytes)
        fixed (byte* bufferPtr = buffer)
        {
            return Generated.render_to_buffer_with_format(nodePtr, viewport, format, bufferPtr, (ulong)buffer.Length);
        }
    }

    /// <summary>
    /// Renders a node and returns the encoded image data as a byte array.
    /// </summary>
    /// <param name="nodeJson">JSON string representing the node to render</param>
    /// <param name="viewport">Viewport settings for rendering</param>
    /// <param name="format">The image format to encode to</param>
    /// <returns>The encoded image data, or null if an error occurred</returns>
    public static byte[]? RenderToBytes(string nodeJson, Viewport viewport, ImageFormat format)
    {
        ulong size = CalculateBufferSize(nodeJson, viewport, format);
        if (size == 0)
        {
            return null;
        }

        byte[] buffer = new byte[size];
        if (!RenderToBuffer(nodeJson, viewport, format, buffer))
        {
            return null;
        }

        return buffer;
    }
}