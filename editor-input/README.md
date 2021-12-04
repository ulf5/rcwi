editor-input
============

A simple library containing a method for accepting input from the editor
specified by the $EDITOR environment variable (like `git` does).

Example
-------

```rust
fn main() {
    let edited_string = editor_input::input_from_editor("placeholder text").unwrap();
    println!("{}", edited_string);
}
```
