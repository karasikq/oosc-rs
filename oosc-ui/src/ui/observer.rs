use oosc_core::utils::Shared;

pub trait Observer<T, E> {
    fn react(&mut self, value: T);
    fn event(&self) -> E;
}

pub trait Notifier<T, E> {
    fn subscribe<O: Observer<T, E> + 'static>(&mut self, observer: Shared<O>);
    fn notify(&mut self, event: E, value: T);
}

pub struct NotifierContainer<E, T>
where
    E: Eq + PartialEq,
{
    observers: Vec<Shared<dyn Observer<T, E>>>,
}

impl<E, T> NotifierContainer<E, T>
where
    E: Eq + PartialEq,
{
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }
}

impl<E, T> Default for NotifierContainer<E, T>
where
    E: Eq + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, E> Notifier<T, E> for NotifierContainer<E, T>
where
    T: Clone,
    E: Eq + PartialEq,
{
    fn subscribe<O: Observer<T, E> + Sized + 'static>(
        &mut self,
        observer: Shared<O>,
    ) {
        self.observers.push(observer);
    }

    fn notify(&mut self, event: E, value: T) {
        self.observers
            .iter_mut()
            .filter(|o| o.read().unwrap().event() == event)
            .for_each(|o| o.write().unwrap().react(value.clone()));
    }
}
