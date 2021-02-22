use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

/// read or read-write mode smart pointer
pub enum ReadWrite<T>
where
    T: Clone,
{
    R(Arc<T>),
    Rw(Arc<T>),
}

impl<T> ReadWrite<T>
where
    T: Clone,
{
    pub fn new(t: T) -> Self {
        ReadWrite::Rw(Arc::new(t))
    }

    pub fn clone_read(&self) -> Self {
        match self {
            ReadWrite::R(arc) => ReadWrite::R(Arc::clone(arc)),
            ReadWrite::Rw(arc) => ReadWrite::R(Arc::clone(arc)),
        }
    }

    pub fn strong_count(&self) -> usize {
        match self {
            ReadWrite::R(arc) => Arc::strong_count(arc),
            ReadWrite::Rw(arc) => Arc::strong_count(arc),
        }
    }
}

impl<T> Deref for ReadWrite<T>
where
    T: Clone,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ReadWrite::R(arc) => arc.deref(),
            ReadWrite::Rw(arc) => arc.deref(),
        }
    }
}

/// only ReadWrite::Rw can deref mut
impl<T> DerefMut for ReadWrite<T>
where
    T: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ReadWrite::R(_) => panic!("ReadWrite::R can not modified"),
            ReadWrite::Rw(arc) => {
                // not clonned before, just use self
                if Arc::strong_count(arc) == 1 {
                    Arc::get_mut(arc).unwrap()
                } else {
                    // if already clonned, clone T and write to it
                    let clonned = <T as Clone>::clone(arc);
                    *arc = Arc::new(clonned);
                    Arc::get_mut(arc).unwrap()
                }
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
        let rw = ReadWrite::new(my);
        assert_eq!(rw.val, 10);
    }

    #[test]
    fn write_test() {
        let my = MyStruct { val: 10 };
        let mut rw = ReadWrite::new(my);

        rw.val = 20;
        assert_eq!(rw.val, 20);
    }

    #[test]
    fn test_clone_read_share_internally() {
        let rw = ReadWrite::new(MyStruct { val: 10 });
        assert_eq!(rw.strong_count(), 1);

        let cloned = rw.clone_read();
        assert_eq!(rw.strong_count(), 2);
        assert_eq!(cloned.strong_count(), 2);
    }

    #[test]
    fn write_after_clone_clone_original_value() {
        let mut rw = ReadWrite::new(MyStruct { val: 10 });
        let clonned = rw.clone_read();

        rw.val = 100;

        assert_eq!(rw.strong_count(), 1);
        assert_eq!(clonned.strong_count(), 1);

        assert_eq!(rw.val, 100);
        assert_eq!(clonned.val, 10);
    }

    #[test]
    fn can_pass_between_threads() {
        let rw = ReadWrite::new(MyStruct { val: 10 });
        let clonned = rw.clone_read();

        let handle = thread::spawn(move || {
            assert_eq!(clonned.val, 10);
        });

        handle.join().unwrap();
    }
}
