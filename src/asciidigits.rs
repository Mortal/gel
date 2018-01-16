pub const DIGITS: [[u8; 35]; 10] = [*b"\
xxxxx\
x...x\
x...x\
x...x\
x...x\
x...x\
xxxxx\
", *b"\
....x\
....x\
....x\
....x\
....x\
....x\
....x\
", *b"\
xxxxx\
....x\
....x\
xxxxx\
x....\
x....\
xxxxx\
", *b"\
xxxxx\
....x\
....x\
xxxxx\
....x\
....x\
xxxxx\
", *b"\
x...x\
x...x\
x...x\
xxxxx\
....x\
....x\
....x\
", *b"\
xxxxx\
x....\
x....\
xxxxx\
....x\
....x\
xxxxx\
", *b"\
xxxxx\
x....\
x....\
xxxxx\
x...x\
x...x\
xxxxx\
", *b"\
xxxxx\
....x\
....x\
....x\
....x\
....x\
....x\
", *b"\
xxxxx\
x...x\
x...x\
xxxxx\
x...x\
x...x\
xxxxx\
", *b"\
xxxxx\
x...x\
x...x\
xxxxx\
....x\
....x\
xxxxx\
"];

macro_rules! draw {
    ( $digit:expr, $cb:expr ) => {
        for i in 0..7 {
            for j in 0..5 {
                if DIGITS[$digit][i*5+j] == b'x' {
                    $cb(i, j);
                }
            }
        }
    }
}

pub fn draw<F: FnMut(usize, usize)>(digit: usize, mut f: F) {
    match digit {
        0 => draw!(0, f),
        1 => draw!(1, f),
        2 => draw!(2, f),
        3 => draw!(3, f),
        4 => draw!(4, f),
        5 => draw!(5, f),
        6 => draw!(6, f),
        7 => draw!(7, f),
        8 => draw!(8, f),
        9 => draw!(9, f),
        _ => panic!("Invalid digit"),
    }
}
