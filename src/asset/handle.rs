#![allow(dead_code)]

use std::cell::RefCell;

use crossbeam_channel::RecvError;

use super::assets::Asset;

pub enum AssetLoadError{
    Failed,
}

pub struct AssetHandle<'a, T: Asset + 'static> {
    state: RefCell<State<'a, T>>,
}

impl<'a, T: Asset + 'static> AssetHandle<'a, T> {
    pub fn new(recv: ReceiveType<'a, T>) -> Self {
        Self {
            state: RefCell::new(State::Processing(recv)),
        }
    }

    /// block until loading completed or failed
    pub fn get_asset(&self) -> Option<&'a T> {
        self.state.borrow_mut().get_asset()
    }
}

pub type ResultType<'a, T> = Result<&'a T, AssetLoadError>;
pub type ReceiveType<'a, T> = crossbeam_channel::Receiver<ResultType<'a, T>>;

enum State<'a, T> 
where
    T: Asset + 'static
{
    Processing(ReceiveType<'a, T>),
    Completed(&'a T),
    Failed(AssetLoadError),
}

impl<'a, T> State<'a, T>
where 
    T: Asset + 'static
{
    pub fn get_asset(&mut self) -> Option<&'a T> {
        match self {
            State::Processing(receive_chan) => {
                let received = receive_chan.recv();
                self.get_asset_from_channel(received)
            }
            State::Completed(asset) => {
                Some(asset)
            }
            State::Failed(_) => {
                None
            }
        }
    }    

    fn get_asset_from_channel(&mut self, result: Result<ResultType<'a, T>, RecvError>) -> Option<&'a T> {
        // check channel receive error
        if result.is_err() {
            *self = State::Failed(AssetLoadError::Failed);
            return None
        }

        let result = result.unwrap();
        match result {
            Ok(asset) => {
                *self = State::Completed(asset);
                Some(asset)
            }
            Err(err) => {
                *self = State::Failed(err);
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::asset::TextAsset;

    use super::*;

    #[test]
    fn create_asset_handle() {
        let (sender, recv) = crossbeam_channel::unbounded();
        let h = AssetHandle::new(recv);

        let asset = Box::leak(Box::new(TextAsset::new("text".to_string())));
        let _ = sender.send(Ok(asset));

        let get = h.get_asset().unwrap();
        assert_eq!(get.text, "text");
    }
}