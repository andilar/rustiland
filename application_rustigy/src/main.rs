// rust-lang

fn main() {
    println!("application rustigy started! ");

    let mut x = 49865547671119i64;
    let mut done = false;

    while !done {
        x += x -5;

        println!("{}", x);

        if x % 13 == 0{
            done = true;
        }
    }
}
