use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

/// write after clone
/// cloned T is never chagned
pub enum SafeCloner<T>
where
    T: Clone,
{
    R(Arc<T>),
    Rw(Arc<T>),
}

impl<T> SafeCloner<T>
where
    T: Clone,
{
    pub fn new(t: T) -> Self {
        SafeCloner::Rw(Arc::new(t))
    }

    pub fn clone_read(&self) -> Self {
        match self {
            SafeCloner::R(arc) => SafeCloner::R(Arc::clone(arc)),
            SafeCloner::Rw(arc) => SafeCloner::R(Arc::clone(arc)),
        }
    }

    pub fn strong_count(&self) -> usize {
        match self {
            SafeCloner::R(arc) => Arc::strong_count(arc),
            SafeCloner::Rw(arc) => Arc::strong_count(arc),
        }
    }
}

impl<T> Deref for SafeCloner<T>
where
    T: Clone,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            SafeCloner::R(arc) => arc.deref(),
            SafeCloner::Rw(arc) => arc.deref(),
        }
    }
}

/// only ReadWrite::Rw can deref mut
impl<T> DerefMut for SafeCloner<T>
where
    T: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            SafeCloner::R(_) => panic!("SafeCloner::R can not modified"),
            SafeCloner::Rw(arc) => {
                // if already clonned, clone T and write to it
                if Arc::strong_count(arc) != 1 {
                    let clonned = <T as Clone>::clone(arc);
                    *arc = Arc::new(clonned);
                }
                Arc::get_mut(arc).unwrap()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[derive(Clone)]
    struct MyStruct {
        val: i32,
    }

    #[test]
    fn test_new() {
        let my = MyStruct { val: 10 };
        let sc = SafeCloner::new(my);
        assert_eq!(sc.val, 10);
    }

    #[test]
    fn write_test() {
        let my = MyStruct { val: 10 };
        let mut sc = SafeCloner::new(my);

        sc.val = 20;
        assert_eq!(sc.val, 20);
    }

    #[test]
    fn test_clone_read_share_internally() {
        let sc = SafeCloner::new(MyStruct { val: 10 });
        assert_eq!(sc.strong_count(), 1);

        let cloned = sc.clone_read();
        assert_eq!(sc.strong_count(), 2);
        assert_eq!(cloned.strong_count(), 2);
    }

    #[test]
    fn write_after_clone() {
        let mut rw = SafeCloner::new(MyStruct { val: 10 });
        let cloned = rw.clone_read();
        assert_eq!(rw.strong_count(), 2);
        assert_eq!(cloned.strong_count(), 2);

        rw.val = 100;

        assert_eq!(rw.strong_count(), 1);
        assert_eq!(cloned.strong_count(), 1);

        assert_eq!(rw.val, 100);
        assert_eq!(cloned.val, 10);

        rw.val = 200;

        assert_eq!(rw.strong_count(), 1);
        assert_eq!(cloned.strong_count(), 1);

        assert_eq!(rw.val, 200);
        assert_eq!(cloned.val, 10);
    }

    #[test]
    fn can_pass_between_threads() {
        let rw = SafeCloner::new(MyStruct { val: 10 });
        let clonned = rw.clone_read();

        let handle = thread::spawn(move || {
            assert_eq!(clonned.val, 10);
        });

        handle.join().unwrap();
    }
}
