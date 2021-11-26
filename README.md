## rcwi

### WIP

Very simple and incomplete TUI for CloudWatch Insights made in Rust.

- First select which log groups you want to search in.
- Edit your query (respects `$EDITOR`).
- Edit time range.
- Run the query.

Uses the default credential chain for AWS credentials.  
So to change region from your config's default you can run `rcwi` with `AWS_REGION` set to something else.

### Installation

Requires Rust, can be installed from [here](https://rustup.rs/)

- Clone repository
- in directory run `cargo install --path .`


### Limitations

This is very much a work in progress and there are lots of limitations.
Most of them will be obvious when using the program, but this might not be:  
Currently requires the query to contain the fields `@message` and `@timestamp`.
Only displays `@message`.
