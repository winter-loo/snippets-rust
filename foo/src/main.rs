fn main() {

}

use std::collections::HashMap;

/// `InputCellId` is a unique identifier for an input cell.
#[derive(Clone, Hash, Copy, Debug, PartialEq, Eq)]
pub struct InputCellId(usize);
/// `ComputeCellId` is a unique identifier for a compute cell.
/// Values of type `InputCellId` and `ComputeCellId` should not be mutually assignable,
/// demonstrated by the following tests:
///
/// ```compile_fail
/// let mut r = react::Reactor::new();
/// let input: react::ComputeCellId = r.create_input(111);
/// ```
///
/// ```compile_fail
/// let mut r = react::Reactor::new();
/// let input = r.create_input(111);
/// let compute: react::InputCellId = r.create_compute(&[react::CellId::Input(input)], |_| 222).unwrap();
/// ```
#[derive(Clone, Hash, Copy, Debug, PartialEq, Eq)]
pub struct ComputeCellId(usize);
#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallbackId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellId {
    Input(InputCellId),
    Compute(ComputeCellId),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RemoveCallbackError {
    NonexistentCell,
    NonexistentCallback,
}

pub struct Reactor<'a, T> {
    // Just so that the compiler doesn't complain about an unused type parameter.
    // You probably want to delete this field.
    input_cells: HashMap<InputCellId, (T, Vec<ComputeCellId>)>,
    compute_cells: HashMap<
        ComputeCellId,
        (
            Vec<CellId>,
            Box<dyn Fn(&[T]) -> T>,
            Vec<CallbackId>,
            Vec<ComputeCellId>,
        ),
    >,
    num_cells: usize,
    // lifetime `'a` ensures that the closures remain valid for the lifetime of the Reactor
    // instance
    callbacks: HashMap<CallbackId, Box<dyn 'a + FnMut(T)>>,
    callback_id_gen: usize,
}

// You are guaranteed that Reactor will only be tested against types that are Copy + PartialEq.
impl<'a, T: Copy + PartialEq> Reactor<'a, T> {
    pub fn new() -> Self {
        Reactor {
            input_cells: HashMap::new(),
            compute_cells: HashMap::new(),
            num_cells: 0,
            callbacks: HashMap::new(),
            callback_id_gen: 0,
        }
    }

    // Creates an input cell with the specified initial value, returning its ID.
    pub fn create_input(&mut self, initial: T) -> InputCellId {
        let id = InputCellId(self.num_cells);
        self.num_cells += 1;
        self.input_cells.insert(id, (initial, vec![]));
        id
    }

    // Creates a compute cell with the specified dependencies and compute function.
    // The compute function is expected to take in its arguments in the same order as specified in
    // `dependencies`.
    // You do not need to reject compute functions that expect more arguments than there are
    // dependencies (how would you check for this, anyway?).
    //
    // If any dependency doesn't exist, returns an Err with that nonexistent dependency.
    // (If multiple dependencies do not exist, exactly which one is returned is not defined and
    // will not be tested)
    //
    // Notice that there is no way to *remove* a cell.
    // This means that you may assume, without checking, that if the dependencies exist at creation
    // time they will continue to exist as long as the Reactor exists.
    pub fn create_compute<F: Fn(&[T]) -> T + 'static>(
        &mut self,
        dependencies: &[CellId],
        compute_func: F,
    ) -> Result<ComputeCellId, CellId> {
        let id = ComputeCellId(self.num_cells);

        for dep in dependencies {
            match dep {
                CellId::Input(i) => match self.input_cells.get_mut(i) {
                    Some((_, dependents)) => {
                        dependents.push(id);
                    }
                    None => return Err(*dep),
                },
                CellId::Compute(c) => match self.compute_cells.get_mut(c) {
                    None => return Err(*dep),
                    Some((_, _, _, dependents)) => {
                        dependents.push(id);
                    }
                },
            }
        }
        self.num_cells += 1;
        self.compute_cells.insert(
            id,
            (
                dependencies.to_vec(),
                Box::new(compute_func),
                vec![],
                vec![],
            ),
        );
        Ok(id)
    }

    // Retrieves the current value of the cell, or None if the cell does not exist.
    //
    // You may wonder whether it is possible to implement `get(&self, id: CellId) -> Option<&Cell>`
    // and have a `value(&self)` method on `Cell`.
    //
    // It turns out this introduces a significant amount of extra complexity to this exercise.
    // We chose not to cover this here, since this exercise is probably enough work as-is.
    pub fn value(&self, id: CellId) -> Option<T> {
        match id {
            CellId::Input(i) => self.input_cells.get(&i).map(|(t, _)| *t),
            CellId::Compute(c) => self
                .compute_cells
                .get(&c)
                .map(|(deps, compute_func, _, _)| {
                    let deps: Option<Vec<_>> = deps.iter().map(|id| self.value(*id)).collect();
                    deps.map(|v| (compute_func)(&v))
                })
                .flatten(),
        }
    }

    // Sets the value of the specified input cell.
    //
    // Returns false if the cell does not exist.
    //
    // Similarly, you may wonder about `get_mut(&mut self, id: CellId) -> Option<&mut Cell>`, with
    // a `set_value(&mut self, new_value: T)` method on `Cell`.
    //
    // As before, that turned out to add too much extra complexity.
    pub fn set_value(&mut self, id: InputCellId, new_value: T) -> bool {
        self.input_cells
            .get_mut(&id)
            .map_or(false, |(value, dependents)| {
                if *value != new_value {
                    *value = new_value;
                    dependents.iter().for_each(|cid| {
                        self.compute_cells.get(cid).map(|(deps, compute_func, callbacks, _)| {
                            callbacks.iter().for_each(|cb_id| {
                                self.callbacks.get_mut(cb_id).map(|cb_action| {
                                    let deps: Option<Vec<_>> = deps.iter().map(|id| self.value(*id)).collect();
                                    let new_compute_value = deps.map(|v| (compute_func)(&v));
                                    (cb_action)(new_compute_value.unwrap());
                                });
                            });
                        });
                    });
                }
                true
            })
    }

    // Adds a callback to the specified compute cell.
    //
    // Returns the ID of the just-added callback, or None if the cell doesn't exist.
    //
    // Callbacks on input cells will not be tested.
    //
    // The semantics of callbacks (as will be tested):
    // For a single set_value call, each compute cell's callbacks should each be called:
    // * Zero times if the compute cell's value did not change as a result of the set_value call.
    // * Exactly once if the compute cell's value changed as a result of the set_value call.
    //   The value passed to the callback should be the final value of the compute cell after the
    //   set_value call.
    pub fn add_callback<F: FnMut(T) + 'a>(
        &mut self,
        id: ComputeCellId,
        callback: F,
    ) -> Option<CallbackId> {
        self.compute_cells.get_mut(&id).map(|(_, _, callbacks, _)| {
            let id = CallbackId(self.callback_id_gen);
            self.callback_id_gen += 1;
            self.callbacks.insert(id, Box::new(callback));
            callbacks.push(id);
            id
        })
    }

    // Removes the specified callback, using an ID returned from add_callback.
    //
    // Returns an Err if either the cell or callback does not exist.
    //
    // A removed callback should no longer be called.
    pub fn remove_callback(
        &mut self,
        cell: ComputeCellId,
        callback: CallbackId,
    ) -> Result<(), RemoveCallbackError> {
        match self.compute_cells.get_mut(&cell) {
            Some((.., callbacks, _)) => {
                let found = callbacks.iter().enumerate().find(|(_, n)| **n == callback);
                if found.is_none() {
                    return Err(RemoveCallbackError::NonexistentCallback);
                }
                let found = found.unwrap();
                callbacks.remove(found.0);
            }
            None => return Err(RemoveCallbackError::NonexistentCell),
        }
        self.callbacks.remove(&callback);
        Ok(())
    }
}
