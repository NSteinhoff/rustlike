//! Pushdown Automaton based engine using dynamic dispatch through trait objects
type BoxedState<D, A> = Box<dyn State<Data = D, Action = A>>;

pub struct Event;

pub trait State: std::fmt::Debug {
    type Data;
    type Action;

    fn render(&self) {
        println!("STATE={:?}: rendering", self);
    }
    fn interpret(&self, event: Event) -> Self::Action;
    fn update(
        &self,
        data: &mut Self::Data,
        action: Self::Action,
    ) -> Transition<Self::Data, Self::Action>;
}

#[derive(Debug)]
pub enum Transition<D, A> {
    Continue,
    Break,
    Next(BoxedState<D, A>),
    Replace(BoxedState<D, A>),
}

#[derive(Debug)]
pub struct Engine<D, A> {
    stack: Vec<BoxedState<D, A>>,
}

impl<D, A> Engine<D, A>
where
    D: std::fmt::Debug,
    A: std::fmt::Debug,
{
    pub fn new(start: BoxedState<D, A>) -> Self {
        Engine {
            stack: vec![start],
        }
    }

    pub fn run(&mut self, mut data: D) -> D {
        while let Some(state) = self.stack.pop() {
            println!("ENGINE: stack = {:?}", self.stack);

            println!("ENGINE: state = {:?}", state);

            state.render();

            let action = state.interpret(self.next_event());
            println!("ENGINE: action = {:?}", action);

            let transition = state.update(&mut data, action);
            println!("ENGINE: transition = {:?}", transition);

            match transition {
                Transition::Continue => {
                    self.stack.push(state);
                },
                Transition::Break => {},
                Transition::Next(next_state) => {
                    self.stack.push(state);
                    self.stack.push(next_state);
                }
                Transition::Replace(new_state) => {
                    self.stack.push(new_state);
                }
            }
        }

        println!("ENGINE: stack empty");
        data
    }

    fn next_event(&mut self) -> Event {
        Event {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct StateOne;

    #[derive(Debug)]
    struct StateTwo;

    impl StateOne {
        fn boxed() -> Box<Self> {
            Box::new(Self {})
        }
    }

    impl State for StateOne {
        type Data = String;
        type Action = i32;

        fn interpret(&self, _event: Event) -> Self::Action {
            3
        }
        fn update(
            &self,
            _data: &mut Self::Data,
            action: Self::Action,
        ) -> Transition<Self::Data, Self::Action> {
            match action {
                1 => Transition::Continue,
                2 => Transition::Next(StateTwo::boxed()),
                3 => Transition::Replace(StateTwo::boxed()),
                _ => Transition::Break,
            }
        }
    }

    impl StateTwo {
        fn boxed() -> Box<Self> {
            Box::new(Self {})
        }
    }

    impl State for StateTwo {
        type Data = String;
        type Action = i32;

        fn interpret(&self, _event: Event) -> Self::Action {
            5
        }
        fn update(
            &self,
            _data: &mut Self::Data,
            action: Self::Action,
        ) -> Transition<Self::Data, Self::Action> {
            match action {
                1 => Transition::Continue,
                2 => Transition::Next(StateOne::boxed()),
                3 => Transition::Replace(StateOne::boxed()),
                _ => Transition::Break,
            }
        }
    }


    #[test]
    fn create_engine() {
        let engine = Engine::new(StateOne::boxed());
        println!("Engine: {:?}", engine);
        let Engine { stack, .. } = engine;
        assert_eq!(stack.len(), 1)
    }

    #[test]
    fn run_engine() {
        let mut engine = Engine::new(StateOne::boxed());
        println!("Engine: {:?}", engine);

        let data = String::from("some data");

        let result = engine.run(data);

        assert_eq!(result, "some data");
    }
}
