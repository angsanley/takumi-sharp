using System.Text.Json;
using TakumiSharp.Models;

namespace TakumiSharp;

public static class Takumi
{
  public static void LoadFont(string fontPath) => Internal.Renderer.LoadFont(fontPath);
  public static void LoadFont(ReadOnlySpan<byte> fontData) => Internal.Renderer.LoadFont(fontData);

  public static byte[] Render(
    NodeKind node,
    int? width = null,
    int? height = null,
    float fontSize = 16f,
    float devicePixelRatio = 1f,
    ImageFormat format = ImageFormat.WebP)
  {
    string nodeJson = JsonSerializer.Serialize(node);
    return Internal.Renderer.Render(
      nodeJson,
      width: width,
      height: height,
      fontSize: fontSize,
      devicePixelRatio: devicePixelRatio,
      format: format.ToInternalImageFormat()
    );
  }

}
