using TakumiSharp.Bindings;

namespace TakumiSharp.Models;

public enum ImageFormat
{
  Png,
  Jpeg,
  Gif,
  WebP,
  Pnm,
  Tiff,
  Tga,
  Dds,
  Bmp,
  Ico,
  Hdr,
  OpenExr,
  Farbfeld,
  Avif,
  Qoi
}

internal static class ImageFormatExtensions
{
  internal static Bindings.ImageFormat ToInternalImageFormat(this ImageFormat format) => format switch
  {
    ImageFormat.Png => Bindings.ImageFormat.Png,
    ImageFormat.Jpeg => Bindings.ImageFormat.Jpeg,
    ImageFormat.Gif => Bindings.ImageFormat.Gif,
    ImageFormat.WebP => Bindings.ImageFormat.WebP,
    ImageFormat.Pnm => Bindings.ImageFormat.Pnm,
    ImageFormat.Tiff => Bindings.ImageFormat.Tiff,
    ImageFormat.Tga => Bindings.ImageFormat.Tga,
    ImageFormat.Dds => Bindings.ImageFormat.Dds,
    ImageFormat.Bmp => Bindings.ImageFormat.Bmp,
    ImageFormat.Ico => Bindings.ImageFormat.Ico,
    ImageFormat.Hdr => Bindings.ImageFormat.Hdr,
    ImageFormat.OpenExr => Bindings.ImageFormat.OpenExr,
    ImageFormat.Farbfeld => Bindings.ImageFormat.Farbfeld,
    ImageFormat.Avif => Bindings.ImageFormat.Avif,
    ImageFormat.Qoi => Bindings.ImageFormat.Qoi,
    _ => Bindings.ImageFormat.Png
  };
}