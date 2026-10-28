- # Queries
	- Each box below is a Logseq query that the build evaluates against this garden.
- Pages tagged with [[Tools]]
	- {{query (page-tags [[Tools]])}}
	  query-properties:: [:page :alias :tags]
	  query-sort-by:: page
	  query-sort-desc:: false
- Blocks under tmux that mention its prefix key
	- {{query (and [[Tools/tmux]] "C-b")}}
- Journal blocks from 2026-10-19 to 2026-10-21
	- {{query (between [[2026-10-19]] [[2026-10-21]])}}
- A query that matches nothing
	- {{query [[Tools/No Such Page]]}}
- A query that does not parse
	- {{query (and [[Tools/tmux]] "x"}}
- A query with a form the build does not support
	- {{query (between created-at -7d today)}}
- A collapsed advanced query
	- #+BEGIN_QUERY
	  {:title [:h3 "Blocks that mention tmux"]
	   :query [[Tools/tmux]]
	   :collapsed? true}
	  #+END_QUERY
- An advanced query in Datalog
	- #+BEGIN_QUERY
	  {:title "Open tasks"
	   :query [:find (pull ?b [*]) :where [?b :block/marker "TODO"]]}
	  #+END_QUERY
- A page named only inside an advanced query
	- #+BEGIN_QUERY
	  {:title "Query target" :query [[Tools/Query Target]]}
	  #+END_QUERY
