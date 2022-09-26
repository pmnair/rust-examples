
use asyncworkers::*;

fn main() {
    let mut w = Workers::new(3);
    let arr = vec![1,2,3,4,5,6,7,8,9,10];
    let var = arr.clone();
    w.execute(move || {
        println!("Executing work 1!");
        for i in &var[..4] {
            println!("Printing {}", i);
        }
    });

    w.execute(move || {
        println!("Executing work 2");
        for i in &arr[5..9] {
            println!("Printing {}", i);
        }
    });
}
