
use macros;

fn main() {

    macros::print_ast!(
        {* 4 7}
    );

    macros::interp!(
        {declare
            ([[a : u32] 4] [[b : u32] 7]) : u32
            in
            {+ a b}}
    );

    macros::print_ast!(
        {if true "5" "7"}
    );

    macros::interp!(
        {{proc ([a : u32]) : u32 {+ a a}} 7}
    );

    macros::interp!(
        {sq 7}
    );

    // macros::interp!(
    //     {fib 7}
    // );

    macros::interp!(
        {strlen "Hello!"}
    );

}


fn sq(x: i32) -> i32 {
    x * x
}

// fn fib(x: i64) -> i64 {
//     match x {
//         0 => 0,
//         1 => 1,
//         x => fib(x-2) + fib(x-1),
//     }
// }

fn strlen(s: &str) -> usize {
    s.len()
}