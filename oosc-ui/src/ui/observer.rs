use oosc_core::utils::Shared;

pub trait Observer<T> {
    fn react(&mut self, value: T);
}

pub trait Notifier<T> {
    fn subscribe<O: Observer<T> + 'static>(&mut self, observer: Shared<O>);
    fn notify(&mut self, value: T);
}

pub struct NotifierContainer<T> {
    observers: Vec<Shared<dyn Observer<T>>>,
}

impl<T> NotifierContainer<T> {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }
}

impl<T> Default for NotifierContainer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Notifier<T> for NotifierContainer<T>
where
    T: Clone,
{
    fn subscribe<O: Observer<T> + Sized + 'static>(&mut self, observer: Shared<O>) {
        self.observers.push(observer);
    }

    fn notify(&mut self, value: T) {
        self.observers
            .iter_mut()
            .for_each(|o| o.write().unwrap().react(value.clone()));
    }
}
