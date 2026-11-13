(ns gardener.relationships
  (:require ["fs" :as fs]
            ["path" :as path]
            [datascript.core :as d]
            [logseq.graph-parser.cli :as gp]))

(let [[garden corpus-path output] (vec *command-line-args*)
      corpus (js->clj (js/JSON.parse (str (fs/readFileSync corpus-path))) :keywordize-keys true)
      files (mapv (fn [{:keys [id input]}] {:file/path (path/resolve garden id) :file/content input}) corpus)
      {:keys [conn]} (gp/parse-graph garden {:files files :verbose false})
      db @conn
      entities (map first (d/q '[:find (pull ?e [*]) :where [?e :block/uuid]] db))
      by-id (into {} (map (juxt :db/id identity) entities))
      blocks (filter :block/content entities)
      ordered (sort-by (juxt #(get-in by-id [(get-in % [:block/page :db/id]) :block/name])
                            #(get-in % [:block/meta :start-pos]) :block/content :db/id) blocks)
      labels (into {} (map-indexed (fn [i e] [(:db/id e) (str "block:" i)]) ordered))
      label (fn [ref] (let [id (:db/id ref)] (or (:block/name (get by-id id)) (get labels id) (some-> (get by-id id) :block/uuid str) "unresolved")))
      pages (->> entities (filter :block/name)
                 (map (fn [e] {:name (:block/name e)
                               :original-name (:block/original-name e)
                               :file (when-let [f (:block/file e)]
                                       (path/relative garden (:file/path (d/entity db (:db/id f)))))
                               :aliases (vec (sort (map label (:block/alias e))))
                               :namespace (when (:block/namespace e) (label (:block/namespace e)))
                               :properties (:block/properties e)
                               :journal? (boolean (:block/journal? e))
                               :journal-day (:block/journal-day e)}))
                 (sort-by :name) vec)
      projected (mapv (fn [e] {:id (get labels (:db/id e))
                              :page (label (:block/page e))
                              :content (:block/content e)
                              :parent (label (:block/parent e))
                              :left (label (:block/left e))
                              :refs (vec (sort (map label (:block/refs e))))
                              :path-refs (vec (sort (map label (:block/path-refs e))))
                              :properties (:block/properties e)}) ordered)]
  (fs/writeFileSync output (js/JSON.stringify (clj->js {:pages pages :blocks projected}) nil 2)))
