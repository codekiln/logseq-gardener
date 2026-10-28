- yazi cards
	- {{query (and [[yazi]] [[card]])}}
- CLI tools
	- {{query (page-tags [[CLI/Tool]])}}
- yazi and Rust
	- {{query (and [[yazi]] [[Rust]])}}
- tmux tasks
	- {{query (and (task TODO DOING) [[tmux]])}}
- Last 30 days
	- {{query (between -30d today)}}
- Text
	- {{query "lightweight database"}}
- Proxy pages
	- {{query (page-property logseq-entity [[Logseq/Entity/Proxy/Page]])}}
- Collapsed tmux
	- #+BEGIN_QUERY
	  {:title "Blocks that mention tmux"
	   :query [[tmux]]
	   :collapsed? true}
	  #+END_QUERY
- Open tmux
	- {{query [[tmux]]}}
- Broken
	- {{query (and [[tmux]]}}
