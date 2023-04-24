use ohkami_lib::Global;

static S: Global<String> = Global::new(|| String::from("Hello, world!"));
static A: Global<String> = Global::;

fn main() {
    let s_ref: &String = &S;
    println!("{s_ref}");
}
