using TakumiSharp;
using TakumiSharp.Models;

Takumi.LoadFont("font.ttf");

var imageData = Takumi.Render(
    node: new ContainerNode
    {
      Children = [
        new TextNode
        {
          Text = "Hello, TakumiSharp!",
          Style = new Style() {
            TextAlign = "center",
            FontSize = 100
          }
        }
      ],
      Style = new Style() {
        Height = "100%",
        Width = "100%",
        Display = "flex",
        JustifyContent = "center",
        AlignItems = "center",
        BackgroundColor = "#FFFFFF"
      }
    },
    width: 1920,
    height: 1080,
    format: ImageFormat.Png
);

await File.WriteAllBytesAsync("output.png", imageData);