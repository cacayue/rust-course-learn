// 修复错误
fn main() {
    let x = define_x();
    println!("{}, world", x); 
}

fn define_x() -> String{
  let x = "hello".to_string();
  x
}