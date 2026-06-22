mod model;

fn main() {
    let all = model::load_all();
    println!("codez: {} entries loaded", all.len());
}
