mod app;
mod model;
mod theme;

fn main() {
    let all = model::load_all();
    println!("codez: {} entries loaded", all.len());
}
