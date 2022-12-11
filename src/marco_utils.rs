macro_rules! do_while {
    (($cond:expr)$body:block) => {
        loop {
            let res = $body;
            if !$cond {
                break res;
            }
        }
    };
}

pub(crate) use do_while;
