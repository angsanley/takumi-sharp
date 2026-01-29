using TakumiSharp;

Takumi.LoadFont("font.ttf");

string nodeJson = """
    {
        "type": "text",
        "children": null,
        "text": "Hello, Takumi!",
        "props": null,
        "key": null
    }
    """;

Takumi.RenderToFile(
    nodeJson,
    "hello_takumi.png",
    width: 200,
    height: 100,
    fontSize: 24f
);

Console.WriteLine("Saved to hello_takumi.png");