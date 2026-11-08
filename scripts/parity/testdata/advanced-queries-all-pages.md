- Every page, sorted by its `tags` property, showing the first 1,000: the worst case for an advanced query.
- #+BEGIN_QUERY
  {:title [:h2 "Every page"]
   :query [:find (pull ?p [*])
           :where
           [?p :block/name]]
   :result-transform (fn [rows] (sort-by (fn [row] (str (get-in row [:block/properties :tags]))) rows))
   :view (fn [rows]
           (into ["ul"]
                 (map (fn [row]
                        ["li" ["a" {:href (str "#/page/" (clojure.string/replace (:block/name row) "/" "%2F"))}
                               (:block/original-name row)]]) (take 1000 rows))))
   :table-view? false}
  #+END_QUERY
