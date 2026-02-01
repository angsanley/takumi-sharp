use takumi::{
    GlobalContext, image::ImageFormat, layout::{Viewport, node::{ContainerNode, NodeKind, TextNode}}, rendering::{RenderOptionsBuilder, render}
};

fn main() {
    // Create a node tree with `ContainerNode` and `TextNode`
    let mut node = NodeKind::Container(ContainerNode {
        children: Some(Box::from([NodeKind::Text(TextNode {
            text: "Hello, world!".to_string(),
            style: None, // Construct with `StyleBuilder`
            tw: None,    // Tailwind properties
            preset: None,
        })])),
        preset: None,
        style: None,
        tw: None, // Tailwind properties
    });
    // let mut node = NodeKind::Container(
    //     ContainerNode {
    //         children: None,
    //         preset: None,
    //         style: Some(Style {
    //             background_color: CssValue::Value(Some(ColorInput::Value(Color::white()))),
    //             width: CssValue::Value(Length::Px(630.0)),
    //             height: CssValue::Value(Length::Px(630.0)),
    //             ..Default::default()
    //         }),
    //         tw: None
    //     }
    // );

    let node: NodeKind = match serde_json::from_str::<NodeKind>("{\r\n  \"type\": \"text\",\r\n  \"children\": null,\r\n  \"text\": \"Hidup Jokowi\",\r\n  \"props\": null,\r\n  \"key\": null\r\n}") {
        Ok(v) => v,
        Err(e) => {
            println!("err: {e}");
            todo!();
        },
    };

    // let node_json_str = serde_json::from_value(&node).unwrap();

    // println!("{}", node_json_str);

    // Create a context for storing resources, font caches.
    // You should reuse the context to speed up the rendering.
    let mut global = GlobalContext::default();

    // Load fonts
    global
        .font_context
        .load_and_store(include_bytes!("font.ttf"), None, None)
        .unwrap();

    // Create a viewport
    let viewport = Viewport::new(Some(1200), Some(630));

    // Create render options
    let options = RenderOptionsBuilder::default()
        .viewport(viewport)
        .node(node)
        .global(&global)
        .build()
        .unwrap();

    // Render the layout to an `RgbaImage`
    let image = render(options).unwrap();
    image
        .save_with_format("output2.png", ImageFormat::Png)
        .unwrap();
}
