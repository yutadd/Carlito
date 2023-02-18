pub fn eprintln<T>(str: T)
where
    T: std::fmt::Display,
{
    let _str = format!("{}", str);
    let mut index = 0;
    for i in 0.._str.len() {
        if _str.chars().nth(i).unwrap() == ']' {
            index = i;
            break;
        }
    }
    // https://qiita.com/PruneMazui/items/8a023347772620025ad6
    eprintln!(
        "[\x1b[31m{}\x1b[m]{}\n",
        &_str[1..index],
        &_str[(index + 1).._str.len()]
    );
}
pub fn println<T>(str: T)
where
    T: std::fmt::Display,
{
    let _str = format!("{}", str);
    let mut index = 0;
    for i in 0.._str.len() {
        if _str.chars().nth(i).unwrap() == ']' {
            index = i;
            break;
        }
    }
    // https://qiita.com/PruneMazui/items/8a023347772620025ad6
    println!(
        "[\x1b[32m{}\x1b[m]{}\n",
        &_str[1..index],
        &_str[(index + 1).._str.len()]
    );
}
/*?;明るさや字体;色*/
#[test]
pub fn color_output() {
    eprintln("[output]test_something:test");
    println("[output]test_something:test")
}
