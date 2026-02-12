mod hasher;

fn main() {
    let input1 = "hello world";
    let input2 = "hello worlD";
    let hash1 = hasher::hash_string(input1);
    let hash2 = hasher::hash_string(input2);

    println!("Input1: {}", input1);
    println!("SHA256 1: {}", hash1);
    println!("Input2: {}", input2);
    println!("SHA256 2: {}", hash2);
}

