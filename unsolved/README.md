# not understood

error[E0502]: cannot borrow `*self` as mutable because it is also borrowed as immutable
   --> src/main.rs:138:19
    |
132 |           let handler = self
    |  _______________________-
133 | |             .message_handlers
    | |_____________________________- immutable borrow occurs here
...
138 |               match handler.handle(self, &req.body.payload) {
    |                     ^^^^^^^^------^^^^^^^^^^^^^^^^^^^^^^^^^
    |                     |       |
    |                     |       immutable borrow later used by call
    |                     mutable borrow occurs here

For more information about this error, try `rustc --explain E0502`.
error: could not compile `unsolved` (bin "unsolved") due to 1 previous error
