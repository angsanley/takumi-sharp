using TakumiSharp;
using TakumiSharp.Models;

Takumi.LoadFont("path/to/your/font.ttf");

var imageData = Takumi.Render(
    node: new ContainerNode
    {
      Children = [
        new TextNode
        {
          Text = "Hello, TakumiSharp!",
        }
      ]
    },
    width: 400,
    height: 200,
    format: ImageFormat.Png
);

await File.WriteAllBytesAsync("output.png", imageData);