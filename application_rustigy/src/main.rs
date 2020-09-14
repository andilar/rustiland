fn main() {
    println!("application rustigy started!");

    let mut x = 498;
    let mut done = false;

    while !done {
        x += x -3;

        println!("{}", x);

        if x % 5 == 0{
            done = true;
        }
    }
}
