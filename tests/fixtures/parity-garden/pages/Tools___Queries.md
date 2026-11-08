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
- An advanced query over page properties, sorted newest first
	- #+BEGIN_QUERY
	  {:title [:h3 "Typed pages"]
	   :query [:find (pull ?p [*])
	           :where
	           [?p :block/name]
	           [?p :block/properties ?props]
	           [(get ?props :typed-kind) ?kinds]
	           [(contains? ?kinds "Typed/Kind A")]
	           [(get ?props :typed-public) ?public]
	           [(= ?public true)]]
	   :result-transform (fn [rows]
	                       (sort-by (fn [row] (get-in row [:block/properties :typed-at]))
	                                (fn [a b] (compare b a)) rows))
	   :view (fn [rows]
	           (into ["ul"]
	                 (map (fn [row]
	                        ["li" ["a" {:href (str "#/page/" (clojure.string/replace (:block/name row) "/" "%2F"))}
	                               (:block/original-name row)]]) rows)))}
	  #+END_QUERY
- An advanced query without a view
	- #+BEGIN_QUERY
	  {:title "Pages of the second kind"
	   :query [:find (pull ?p [*])
	           :where
	           [?p :block/name]
	           [?p :block/properties ?props]
	           [(get ?props :typed-kind) ?kinds]
	           [(contains? ?kinds "Typed/Kind B")]]
	   :result-transform (fn [rows] (reverse rows))}
	  #+END_QUERY
- An advanced query that matches nothing
	- #+BEGIN_QUERY
	  {:title "Matches nothing"
	   :query [:find (pull ?p [*])
	           :where
	           [?p :block/name]
	           [?p :block/properties ?props]
	           [(get ?props :typed-kind) ?kinds]
	           [(contains? ?kinds "typed/kind a")]]
	   :view (fn [rows] ["ul" (map (fn [row] ["li" (:block/original-name row)]) rows)])}
	  #+END_QUERY
- An advanced query whose transform throws
	- #+BEGIN_QUERY
	  {:title "Untransformed"
	   :query [:find (pull ?p [*])
	           :where
	           [?p :block/name]
	           [?p :block/properties ?props]
	           [(get ?props :typed-public) ?public]]
	   :result-transform (fn [rows] (sort-by (fn [row] (get-in row [:block/properties :typed-public])) (fn [a b] (compare 1 "a")) rows))}
	  #+END_QUERY
- An advanced query whose view throws
	- #+BEGIN_QUERY
	  {:title "Throws"
	   :query [:find (pull ?p [*])
	           :where
	           [?p :block/name "tools/queries/typed/one"]]
	   :view (fn [rows] (compare "a" 1))}
	  #+END_QUERY
- An advanced query with a function the build does not support
	- #+BEGIN_QUERY
	  {:title "Unsupported"
	   :query [:find (pull ?p [*])
	           :where
	           [?p :block/name "tools/queries/typed/one"]]
	   :result-transform (fn [rows] (partition 2 rows))}
	  #+END_QUERY
