fn main() {
    let mut x = vec![5; 5];
    let y = vec![6; 6];
    let z = x.iter_mut().zip(y.iter()).map(|(elx, ely)| *elx += ely).last();
    println!("{:?}", x);
    println!("{:?}", y);
    println!("{}", z.is_some());
}
