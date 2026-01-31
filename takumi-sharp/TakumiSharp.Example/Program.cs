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
          Tw = "text-4xl font-bold text-center text-blue-600"
        }
      ]
    },
    width: 400,
    height: 200,
    format: ImageFormat.Png
);

await File.WriteAllBytesAsync("output.png", imageData);