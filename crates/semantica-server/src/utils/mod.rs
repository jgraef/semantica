pub mod convert;

macro_rules! bug {
    () => {
        panic!("something went wrong. this is a bug.");
    };
}

pub(crate) use bug;
