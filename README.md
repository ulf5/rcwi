## rcwi

### WIP

Very simple and incomplete TUI for CloudWatch Insights made in Rust.

- First select which log groups you want to search in.
- Edit your query (respects $EDITOR).
- Edit time range.
- Run the query.

### Controls

`q` to quit.  
`hjkl/Arrows` control focus  
`Enter` to select  
`Escape` to go back  
`r` runs the query  


### Limitations

This is very much a work in progress and there are lots of limitations.
Most of them will be obvious when using the program, but this might not be:
Currently requires the query to contain the fields `@message` and `@timestamp`.
Only displays `@message`.
