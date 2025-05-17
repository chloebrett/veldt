use rng::{generate_random_number, generate_random_number_in_range};

fn main() {
    println!("Random number: {}", generate_random_number());
    println!(
        "Random number in range (-1 to 1): {}",
        generate_random_number_in_range(-1.0, 1.0)
    );
}
