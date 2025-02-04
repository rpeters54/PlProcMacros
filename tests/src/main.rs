
use macros;

fn main() {

    println!("{}", macros::interp!(
        {+ 4 7}
    ));

    println!("{}", macros::interp!(
        {declare
            ([a 4] [b 7])
            in
            {+ a b}}
    ));

    println!("{}", macros::interp!(
        {if true 5 7}
    ));

    println!("{}", macros::interp!(
        {{proc (a) {* a a}} 7}
    ));

    println!("{}", macros::interp!(
        {sq 7}
    ));

    println!("{}", macros::interp!(
        {fib 7}
    ));

    println!("{}", macros::interp!(
        {strlen "Hello!"}
    ));
}


fn sq(x: i32) -> i32 {
    x * x
}

fn fib(x: i64) -> i64 {
    match x {
        0 => 0,
        1 => 1,
        x => fib(x-2) + fib(x-1),
    }
}

fn strlen(s: &str) -> usize {
    s.len()
}