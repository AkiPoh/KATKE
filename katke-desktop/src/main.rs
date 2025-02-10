// x is width, y is height

#[derive(Debug)]
struct Panel {
    position1: Vec2,
    position2: Vec2,
    z_index: i32,
}

impl Panel {
    fn new(position1: Vec2, position2: Vec2, z_index: i32) -> Panel {
        Panel { position1, position2, z_index }
    }
}

#[derive(Debug)]
struct Vec2 {
    x: u32, //point in width
    y: u32, // point in height
}

impl Vec2 {
    fn new(x: u32, y: u32) -> Vec2 {
        Vec2 {x, y}
    }
}

fn main() {
    let a_panel = Panel::new(Vec2::new(10, 15), Vec2::new(20, 25), 81);
    print!("{:#?}", a_panel);
}