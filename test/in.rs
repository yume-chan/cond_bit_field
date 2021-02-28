my_macro! {
  let a = 0u8;
  println!("{}", a);

  if cond1 {
    let b = 0u8;
    println!("{}", b);

    for _ in 0..100 {
      let c = 0u8;
      println!("{}", c);
    }
    println!("{:?}", c);
  }

  println!("{:?}", b);
  println!("{:?}", c);
}
