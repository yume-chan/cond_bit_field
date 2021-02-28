let a = 0u8;
println!("{}", a);

let mut b_1: Option<u8> = None;
let mut c_2: Option<Vec<u8>> = None;
if cond1 {
  let b = 0u8;
  println!("{}", b);

  let mut c_1: Vec<u8> = Vec::new();
  for _ in 0..100 {
    let c = 0u8;
    println!("{}", c);

    c_1.push(c);
  }
  let c = c_1;
  println!("{:?}", c);

  b_1 = Some(b);
  c_2 = Some(c);
}

let b = b_1;
let c = c_2;
println!("{:?}", b);
println!("{:?}", c);
