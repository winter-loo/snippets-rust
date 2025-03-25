fn main() {}

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

type ComputeClosure<T> = dyn Fn(&[T]) -> T;

struct ComputeCell<T> {
    depends: Vec<CellId>,
    compute: Box<ComputeClosure<T>>,
    callbacks: Vec<CallbackId>,
    dependents: Vec<ComputeCellId>,
}

pub struct Reactor<'a, T> {
    // Just so that the compiler doesn't complain about an unused type parameter.
    // You probably want to delete this field.
    input_cells: HashMap<InputCellId, (T, Vec<ComputeCellId>)>,
    compute_cells: HashMap<ComputeCellId, ComputeCell<T>>,
    num_cells: usize,
    // lifetime `'a` ensures that the closures remain valid for the lifetime
    // of the Reactor instance
    callbacks: HashMap<CallbackId, Box<dyn 'a + FnMut(T)>>,
    callback_id_gen: usize,
}

impl<T: Copy + PartialEq> Default for Reactor<'_, T> {
    fn default() -> Self {
        Self::new()
    }
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
                    Some(ComputeCell { dependents, .. }) => {
                        dependents.push(id);
                    }
                },
            }
        }
        self.num_cells += 1;
        self.compute_cells.insert(
            id,
            ComputeCell {
                depends: dependencies.to_vec(),
                compute: Box::new(compute_func),
                callbacks: vec![],
                dependents: vec![],
            },
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
            CellId::Compute(c) => self.compute_cells.get(&c).and_then(
                |ComputeCell {
                     depends, compute, ..
                 }| {
                    let deps: Option<Vec<_>> = depends.iter().map(|id| self.value(*id)).collect();
                    deps.map(|v| (compute)(&v))
                },
            ),
        }
    }

    fn get_all_dependents(&self, initial_dependents: &[ComputeCellId]) -> Vec<(ComputeCellId, T)> {
        let mut results: Vec<_> = initial_dependents
            .iter()
            .filter_map(|compute_cell_id| {
                let ccid = CellId::Compute(*compute_cell_id);
                self.value(ccid)
                    .map(|old_value| (*compute_cell_id, old_value))
            })
            .collect();
        let child_dependents: Vec<_> = results
            .iter()
            .filter_map(|dep| {
                self.compute_cells
                    .get(&dep.0)
                    .map(|ComputeCell { dependents, .. }| dependents)
            })
            .flatten()
            .copied()
            .collect();
        if !child_dependents.is_empty() {
            results.extend(self.get_all_dependents(&child_dependents).iter())
        }
        results
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
        let mut set_ok = false;
        // use 'collect' here to avoid 'immutable borrow later used' issue
        let old_values = match self.input_cells.get(&id) {
            None => vec![],
            Some((_, dependents)) => self.get_all_dependents(dependents),
        };

        if let Some((value, _)) = self.input_cells.get_mut(&id) {
            set_ok = true;
            *value = new_value;
        }

        let changed_compute_cells: Vec<_> = old_values
            .iter()
            .filter_map(|(compute_cell_id, old_value)| {
                let ccid = CellId::Compute(*compute_cell_id);
                self.value(ccid).and_then(|new_value| {
                    if *old_value != new_value {
                        Some((*compute_cell_id, new_value))
                    } else {
                        None
                    }
                })
            })
            .collect();

        for (ccid, new_value) in changed_compute_cells {
            if let Some(ComputeCell { callbacks, .. }) = self.compute_cells.get(&ccid) {
                callbacks.iter().for_each(|cb_id| {
                    if let Some(cb_action) = self.callbacks.get_mut(cb_id) {
                        (cb_action)(new_value);
                    }
                });
            }
        }

        set_ok
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
        self.compute_cells
            .get_mut(&id)
            .map(|ComputeCell { callbacks, .. }| {
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
            Some(ComputeCell { callbacks, .. }) => {
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
