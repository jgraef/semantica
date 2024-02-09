pub mod auth;
pub mod error;
pub mod node;
pub mod spell;
pub mod user;

pub trait Links<Id> {
    fn id(&self) -> Id;
}

impl<T: Clone> Links<T> for T {
    fn id(&self) -> T {
        self.clone()
    }
}
