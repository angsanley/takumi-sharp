using System.Text;
using TakumiSharp.Bindings;

namespace TakumiSharp.Internal;

internal class Renderer
{
  /// <summary>
  /// Loads a font from a file path.
  /// </summary>
  /// <param name="fontPath">Path to the font file</param>
  /// <exception cref="FileNotFoundException">Thrown when the font file is not found</exception>
  /// <exception cref="InvalidOperationException">Thrown when the font fails to load</exception>
  public static void LoadFont(string fontPath)
  {
    if (!File.Exists(fontPath))
    {
      throw new FileNotFoundException("Font file not found", fontPath);
    }

    byte[] fontData = File.ReadAllBytes(fontPath);
    LoadFont(fontData);
  }

  /// <summary>
  /// Loads a font from byte data.
  /// </summary>
  /// <param name="fontData">The font file bytes</param>
  /// <exception cref="InvalidOperationException">Thrown when the font fails to load</exception>
  public static unsafe void LoadFont(ReadOnlySpan<byte> fontData)
  {
    bool success;
    fixed (byte* dataPtr = fontData)
    {
      success = NativeBindings.global_font_context_load_and_store(dataPtr, (nuint)fontData.Length);
    }

    if (!success)
    {
      throw new InvalidOperationException("Failed to load font data");
    }
  }

  /// <summary>
  /// Renders a node to a byte array in the specified image format.
  /// </summary>
  /// <param name="nodeJson">JSON string representing the node to render</param>
  /// <param name="width">Viewport width in pixels, or null for auto</param>
  /// <param name="height">Viewport height in pixels, or null for auto</param>
  /// <param name="fontSize">Font size in pixels (default: 16)</param>
  /// <param name="devicePixelRatio">Device pixel ratio (default: 1)</param>
  /// <param name="format">Output image format (default: PNG)</param>
  /// <returns>The encoded image data</returns>
  /// <exception cref="InvalidOperationException">Thrown when rendering fails</exception>
  internal static unsafe byte[] Render(
      string nodeJson,
      int? width = null,
      int? height = null,
      float fontSize = 16f,
      float devicePixelRatio = 1f,
      ImageFormat format = ImageFormat.WebP)
  {
    var viewport = new Viewport
    {
      width = width ?? -1,
      height = height ?? -1,
      font_size = fontSize,
      device_pixel_ratio = devicePixelRatio,
    };

    byte[] nodeBytes = Encoding.UTF8.GetBytes(nodeJson + '\0');

    ulong size;
    fixed (byte* nodePtr = nodeBytes)
    {
      size = NativeBindings.render_calculate_buffer_size_with_format(nodePtr, viewport, format);
    }

    if (size == 0)
    {
      throw new InvalidOperationException("Failed to calculate buffer size for rendering");
    }

    byte[] buffer = new byte[size];
    bool success;
    fixed (byte* nodePtr = nodeBytes)
    fixed (byte* bufferPtr = buffer)
    {
      success = NativeBindings.render_to_buffer_with_format(nodePtr, viewport, format, bufferPtr, size);
    }

    if (!success)
    {
      throw new InvalidOperationException("Failed to render node");
    }

    return buffer;
  }

  /// <summary>
  /// Renders a node and saves it to a file.
  /// </summary>
  /// <param name="nodeJson">JSON string representing the node to render</param>
  /// <param name="outputPath">Path where the image will be saved</param>
  /// <param name="width">Viewport width in pixels, or null for auto</param>
  /// <param name="height">Viewport height in pixels, or null for auto</param>
  /// <param name="fontSize">Font size in pixels (default: 16)</param>
  /// <param name="devicePixelRatio">Device pixel ratio (default: 1)</param>
  /// <param name="format">Output image format (default: PNG)</param>
  /// <exception cref="InvalidOperationException">Thrown when rendering fails</exception>
  internal static void RenderToFile(
      string nodeJson,
      string outputPath,
      int? width = null,
      int? height = null,
      float fontSize = 16f,
      float devicePixelRatio = 1f,
      ImageFormat format = ImageFormat.WebP)
  {
    byte[] data = Render(nodeJson, width, height, fontSize, devicePixelRatio, format);
    File.WriteAllBytes(outputPath, data);
  }

  /// <summary>
  /// Renders a node asynchronously and saves it to a file.
  /// </summary>
  /// <param name="nodeJson">JSON string representing the node to render</param>
  /// <param name="outputPath">Path where the image will be saved</param>
  /// <param name="width">Viewport width in pixels, or null for auto</param>
  /// <param name="height">Viewport height in pixels, or null for auto</param>
  /// <param name="fontSize">Font size in pixels (default: 16)</param>
  /// <param name="devicePixelRatio">Device pixel ratio (default: 1)</param>
  /// <param name="format">Output image format (default: PNG)</param>
  /// <param name="cancellationToken">Cancellation token</param>
  /// <exception cref="InvalidOperationException">Thrown when rendering fails</exception>
  internal static async Task RenderToFileAsync(
      string nodeJson,
      string outputPath,
      int? width = null,
      int? height = null,
      float fontSize = 16f,
      float devicePixelRatio = 1f,
      ImageFormat format = ImageFormat.WebP,
      CancellationToken cancellationToken = default)
  {
    byte[] data = Render(nodeJson, width, height, fontSize, devicePixelRatio, format);
    await File.WriteAllBytesAsync(outputPath, data, cancellationToken);
  }
}