use hashmap::HashMap;
fn main() {
    let _: HashMap<&str, f32> = [
        ("Mercury", 0.4),
        ("Venus", 0.7),
        ("Earth", 1.0),
        ("Mars", 1.5),
    ]
    .iter()
    .cloned()
    .collect();
}
