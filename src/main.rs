use shadow_rs::shadow;

shadow!(build);

fn main() {
    println!("Hello, world! Ver: {}", build::PKG_VERSION);
}
