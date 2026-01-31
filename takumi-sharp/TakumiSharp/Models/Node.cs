using System.Text.Json.Serialization;

namespace TakumiSharp.Models
{
  // NodeKind base class (polymorphic, tagged by "type")
  [JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
  [JsonDerivedType(typeof(ContainerNode), "container")]
  [JsonDerivedType(typeof(TextNode), "text")]
  [JsonDerivedType(typeof(ImageNode), "image")]
  public abstract class NodeKind
  {
  }

  // Container node
  public class ContainerNode : NodeKind
  {
    [JsonPropertyName("preset")]
    public Style? Preset { get; set; }

    [JsonPropertyName("style")]
    public Style? Style { get; set; }

    [JsonPropertyName("children")]
    public List<NodeKind>? Children { get; set; }

    [JsonPropertyName("tw")]
    public TailwindValues? Tw { get; set; }
  }

  // Text node
  public class TextNode : NodeKind
  {
    [JsonPropertyName("preset")]
    public Style? Preset { get; set; }

    [JsonPropertyName("style")]
    public Style? Style { get; set; }

    [JsonPropertyName("text")]
    public string Text { get; set; } = string.Empty;

    [JsonPropertyName("tw")]
    public TailwindValues? Tw { get; set; }
  }

  // Image node
  public class ImageNode : NodeKind
  {
    [JsonPropertyName("preset")]
    public Style? Preset { get; set; }

    [JsonPropertyName("style")]
    public Style? Style { get; set; }

    [JsonPropertyName("src")]
    public string Src { get; set; } = string.Empty;

    [JsonPropertyName("width")]
    public float? Width { get; set; }

    [JsonPropertyName("height")]
    public float? Height { get; set; }

    [JsonPropertyName("tw")]
    public TailwindValues? Tw { get; set; }
  }

  // Placeholder for Style and TailwindValues (to be implemented)
  public class Style { }
  public class TailwindValues { }
}