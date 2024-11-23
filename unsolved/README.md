# the problem

when compile `unsolved` binary, the error below happens:

> error[E0502]: cannot borrow `*self` as mutable because it is also borrowed as immutable
>    --> src/main.rs:138:19
>     |
> 132 |           let handler = self
>     |  _______________________-
> 133 | |             .message_handlers
>     | |_____________________________- immutable borrow occurs here
> ...
> 138 |               match handler.handle(self, &req.body.payload) {
>     |                     ^^^^^^^^------^^^^^^^^^^^^^^^^^^^^^^^^^
>     |                     |       |
>     |                     |       immutable borrow later used by call
>     |                     mutable borrow occurs here
> 
> For more information about this error, try `rustc --explain E0502`.
> error: could not compile `unsolved` (bin "unsolved") due to 1 previous error

## how to solve

Must not add `message_handlers` as the field of `Node`.
Instead, take as a parameter `message_handlers` for `Node.handle` function.
See 'src/bin/solved.rs'.
