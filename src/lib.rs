//! A undo/redo library.
//!
//! It uses the [Command Pattern](https://en.wikipedia.org/wiki/Command_pattern) where the user
//! implements the `UndoCmd` trait for each command and then the commands can be used with the
//! `UndoStack`.
//!
//! The `UndoStack` has two different states, the clean state and the dirty state. The `UndoStack`
//! is in a clean state when there are no more commands that can be redone, otherwise it's in a dirty
//! state.
//!
//! # Example
//! ```
//! use std::rc::Rc;
//! use std::cell::RefCell;
//! use undo::{UndoCmd, UndoStack};
//!
//! /// Pops an element from a vector.
//! #[derive(Clone)]
//! struct PopCmd {
//!     vec: Rc<RefCell<Vec<i32>>>,
//!     e: Option<i32>,
//! }
//!
//! impl UndoCmd for PopCmd {
//!     fn redo(&mut self) {
//!         self.e = self.vec.borrow_mut().pop();
//!     }
//!
//!     fn undo(&mut self) {
//!         self.vec.borrow_mut().push(self.e.unwrap());
//!         self.e = None;
//!     }
//! }
//!
//! // We need to use Rc<RefCell> since all commands are going to mutate the vec.
//! let vec = Rc::new(RefCell::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
//! let mut undo_stack = UndoStack::new()
//!     .on_clean(|| println!("This is called when the stack changes from dirty to clean!"))
//!     .on_dirty(|| println!("This is called when the stack changes from clean to dirty!"));
//!
//! let cmd = PopCmd { vec: vec.clone(), e: None };
//! undo_stack.push(cmd.clone());
//! undo_stack.push(cmd.clone());
//! undo_stack.push(cmd.clone());
//!
//! assert_eq!(vec.borrow().len(), 7);
//!
//! undo_stack.undo(); // on_dirty is going to be called here.
//! undo_stack.undo();
//! undo_stack.undo();
//!
//! assert_eq!(vec.borrow().len(), 10);
//! ```

/// Every command needs to implement the `UndoCmd` trait to be able to be used with the `UndoStack`.
pub trait UndoCmd {
    /// Executes the desired command.
    fn redo(&mut self);
    /// Restores the state as it was before `redo` was called.
    fn undo(&mut self);
}

/// `UndoStack` maintains a stack of `UndoCmd`s that can be undone and redone by using methods
/// on the `UndoStack`.
///
/// `UndoStack` will notice when it's state changes to either dirty or clean, and the user can
/// set methods that should be called for either state change. This is useful for example if
/// you want to automatically enable or disable undo or redo buttons based on there are any
/// more actions that can be undone or redone.
///
/// Note: An empty `UndoStack` is clean, so the first push will not trigger the `on_clean` method.
pub struct UndoStack<'a, T: UndoCmd> {
    stack: Vec<T>,
    len: usize,
    on_clean: Option<Box<FnMut() + 'a>>,
    on_dirty: Option<Box<FnMut() + 'a>>,
}

impl<'a, T: UndoCmd> UndoStack<'a, T> {
    /// Creates a new `UndoStack`.
    pub fn new() -> Self {
        UndoStack {
            stack: Vec::new(),
            len: 0,
            on_clean: None,
            on_dirty: None,
        }
    }

    /// Creates a new `UndoStack` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        UndoStack {
            stack: Vec::with_capacity(capacity),
            len: 0,
            on_clean: None,
            on_dirty: None,
        }
    }

    /// Sets what should happen if the state changes from dirty to clean.
    /// By default the `UndoStack` does nothing when the state changes.
    /// Consumes the `UndoStack` so this method should be called when creating the `UndoStack`.
    pub fn on_clean<F: FnMut() + 'a>(mut self, f: F) -> Self {
        self.on_clean = Some(Box::new(f));
        self
    }

    /// Sets what should happen if the state changes from clean to dirty.
    /// By default the `UndoStack` does nothing when the state changes.
    /// Consumes the `UndoStack` so this method should be called when creating the `UndoStack`.
    pub fn on_dirty<F: FnMut() + 'a>(mut self, f: F) -> Self {
        self.on_dirty = Some(Box::new(f));
        self
    }

    /// Returns true if the state of `UndoStack` is clean.
    pub fn is_clean(&self) -> bool {
        self.len == self.stack.len()
    }

    /// Returns true if the state of `UndoStack` is dirty.
    pub fn is_dirty(&self) -> bool {
        !self.is_clean()
    }

    /// Pushes a `UndoCmd` to the top of the `UndoStack` and executes its `redo()` method.
    /// This pops off all `UndoCmd`s that is above the active `UndoCmd` from the `UndoStack`.
    pub fn push(&mut self, mut cmd: T) {
        let is_dirty = self.is_dirty();
        // Pop off all elements after len from stack.
        self.stack.truncate(self.len);
        cmd.redo();
        self.stack.push(cmd);
        self.len = self.stack.len();
        // State is always clean after a push, check if it was dirty before.
        if is_dirty {
            if let Some(ref mut f) = self.on_clean {
                f();
            }
        }
    }

    /// Calls the `redo` method for the active `UndoCmd` and sets the next `UndoCmd` as the new
    /// active `UndoCmd`. Calling this method when the state is clean does nothing.
    pub fn redo(&mut self) {
        if self.len < self.stack.len() {
            let is_dirty = self.is_dirty();
            {
                let ref mut cmd = self.stack[self.len];
                cmd.redo();
            }
            self.len += 1;
            // Check if stack went from dirty to clean.
            if is_dirty && self.is_clean() {
                if let Some(ref mut f) = self.on_clean {
                    f();
                }
            }
        }
    }

    /// Calls the `undo` method for the active `UndoCmd` and sets the previous `UndoCmd` as the
    /// new active `UndoCmd`.
    pub fn undo(&mut self) {
        if self.len != 0 {
            let is_clean = self.is_clean();
            self.len -= 1;
            {
                let ref mut cmd = self.stack[self.len];
                cmd.undo();
            }
            // Check if stack went from clean to dirty.
            if is_clean && self.is_dirty() {
                if let Some(ref mut f) = self.on_dirty {
                    f();
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use {UndoStack, UndoCmd};

    #[test]
    fn pop() {
        use std::rc::Rc;
        use std::cell::{Cell, RefCell};

        /// Pops an element from a vector.
        #[derive(Clone)]
        struct PopCmd {
            vec: Rc<RefCell<Vec<i32>>>,
            e: Option<i32>,
        }

        impl UndoCmd for PopCmd {
            fn redo(&mut self) {
                self.e = self.vec.borrow_mut().pop();
            }

            fn undo(&mut self) {
                self.vec.borrow_mut().push(self.e.unwrap());
                self.e = None;
            }
        }

        let a = Cell::new(0);
        let b = Cell::new(0);
        // We need to use Rc<RefCell> since all commands are going to mutate the vec.
        let vec = Rc::new(RefCell::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
        let mut undo_stack = UndoStack::with_capacity(3)
            .on_clean(|| a.set(1))
            .on_dirty(|| b.set(1));

        let cmd = PopCmd { vec: vec.clone(), e: None };
        undo_stack.push(cmd.clone());
        undo_stack.push(cmd.clone());
        undo_stack.push(cmd.clone());
        undo_stack.push(cmd.clone());
        undo_stack.push(cmd.clone());
        undo_stack.push(cmd.clone());
        undo_stack.push(cmd.clone());
        undo_stack.push(cmd.clone());
        undo_stack.push(cmd.clone());
        undo_stack.push(cmd.clone());

        assert!(vec.borrow().is_empty());

        assert_eq!(b.get(), 0);
        undo_stack.undo();
        assert_eq!(b.get(), 1);
        b.set(0);

        undo_stack.undo();
        undo_stack.undo();
        undo_stack.undo();
        undo_stack.undo();
        undo_stack.undo();
        undo_stack.undo();
        undo_stack.undo();
        undo_stack.undo();
        undo_stack.undo();

        assert_eq!(vec, Rc::new(RefCell::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9])));

        assert_eq!(a.get(), 0);
        undo_stack.push(cmd.clone());
        assert_eq!(a.get(), 1);
        a.set(0);

        assert_eq!(vec, Rc::new(RefCell::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8])));

        assert_eq!(b.get(), 0);
        undo_stack.undo();
        assert_eq!(b.get(), 1);
        b.set(0);

        assert_eq!(vec, Rc::new(RefCell::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9])));

        assert_eq!(a.get(), 0);
        undo_stack.redo();
        assert_eq!(a.get(), 1);
        a.set(0);

        assert_eq!(vec, Rc::new(RefCell::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8])));
    }
}
