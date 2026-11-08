- The pages that say `logseq-entity:: [[Logseq/Entity/Proxy/Page]]`, in reverse name order, written in the form of gitpa's homepage query.
- #+BEGIN_QUERY
  {:title [:h2 "Proxy pages"]
   :query [:find (pull ?p [*])
           :where
           [?p :block/name]
           [?p :block/properties ?props]
           [(get ?props :logseq-entity) ?entities]
           [(contains? ?entities "Logseq/Entity/Proxy/Page")]]
   :result-transform (fn [rows] (sort-by (fn [row] (:block/original-name row))
                                         (fn [a b] (compare b a)) rows))
   :view (fn [rows]
           (into ["ul"]
                 (map (fn [row]
                        ["li" ["a" {:href (str "#/page/" (clojure.string/replace (:block/name row) "/" "%2F"))}
                               (:block/original-name row)]]) rows)))
   :table-view? false}
  #+END_QUERY
