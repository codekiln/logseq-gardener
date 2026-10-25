tags:: [[Tools]]

- Code blocks and tables as the garden writes them.
	- Loop labels in Rust:
	  ```rust
	  fn loop_labels_to_disambiguate_multiple_loops() {
	      println!("Loop labels to disambiguate multiple loops:");
	      let mut count = 0;
	      'counting_up: loop {
	          println!("count = {count}");
	          let mut remaining = 10;
	  
	          loop {
	              println!("remaining = {remaining}");
	              if remaining == 9 {
	                  break;
	              }
	              if count == 2 {
	                  break 'counting_up;
	              }
	              remaining -= 1;
	          }
	  
	          count += 1;
	      }
	      println!("End count = {count}");
	  }
	  ```
	- | Context | Meaning |
	  |---|:-:|
	  | `'label: loop` | loop label |
	  | `break 'label` | labeled control flow |
	  | `&'a T` | lifetime parameter |
- Shell and Python:
	- ~~~bash
	  # Build the site
	  mise run build:release
	  lsg publish --graph "$HOME/garden" --output site
	  ~~~
	- ```python
	  def batch(items, size=8):
	      """Yield lists of at most size items."""
	      for start in range(0, len(items), size):
	          yield items[start:start + size]
	  ```
	- ```mermaid
	  graph TD
	    A[Request] --> B{Batch?}
	  ```
- #+BEGIN_EXAMPLE
  An example block keeps its text and wraps long lines instead of scrolling sideways like fenced code does.
  #+END_EXAMPLE
