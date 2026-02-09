# TakumiSharp

C# bindings for [Takumi](https://github.com/kane50613/takumi), a high-performance image rendering library powered by Rust. Build beautiful images programmatically using a component-based approach with Tailwind CSS-like styling.

## Installation

Install the NuGet packages:

```bash
dotnet add package TakumiSharp
dotnet add package TakumiSharp.Native
```

## Quick Start

```csharp
using TakumiSharp;
using TakumiSharp.Models;

Takumi.LoadFont("./font.ttf");

var byteResult = Takumi.Render(
    node: new ContainerNode {
        Children = [
            new TextNode {
                Text = "Hello World",
                Tw = "text-3xl"
            }
        ],
        Tw = "w-full h-full flex flex-col bg-white items-center justify-center"
    },
    width: 1920,
    height: 1080,
    fontSize: 16,
    devicePixelRatio: 1,
    format: ImageFormat.Png
);

// save to file
File.WriteAllBytes("output.png", byteResult);
```

## Available Nodes

- **ContainerNode** - A flex container for grouping and laying out child nodes
- **TextNode** - Renders text with customizable styling
- **ImageNode** - Displays images from URLs or local paths

## Styling

TakumiSharp uses Tailwind CSS utility classes for styling. Apply styles using the `Tw` property on any node:

```csharp
new ContainerNode {
    Tw = "flex flex-row gap-4 p-8 bg-gradient-to-r from-blue-500 to-purple-500",
    Children = [
        new TextNode { Text = "Styled Text", Tw = "text-white text-2xl font-bold" }
    ]
}
```

## License

MIT
