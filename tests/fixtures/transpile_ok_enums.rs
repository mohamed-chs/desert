enum Color {
    Red,
    Green,
    Blue,
}
enum Shape {
    Circle(f64),
    Rect(f64, f64),
}
fn main() {
    let c = Color::Red;
    let s = Shape::Circle(3.14);
}

