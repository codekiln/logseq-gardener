;; Parse a graph folder with Logseq's file graph parser, reading its logseq/config.edn as
;; Logseq does, and write its pages and blocks as JSON.
;;
;; Usage: nbb-logseq --classpath <graph-parser/src:db/src:common/src> graph.cljs <graph> <output>
;;
;; The files are the ones logseq.graph-parser.cli/parse-graph reads, named by their path
;; relative to the graph folder, as Logseq desktop names them (frontend.fs/get-files).
;; Pages carry the fields of the current-og-relationships experiment, so
;; scripts/og_relationship_fixtures.py can project them. Blocks are listed page by page in
;; outline order.
(ns gardener.graph
  (:require ["fs" :as fs]
            ["path" :as path]
            [clojure.edn :as edn]
            [datascript.core :as d]
            [logseq.common.config :as common-config]
            [logseq.common.graph :as common-graph]
            [logseq.graph-parser :as graph-parser]
            [logseq.graph-parser.cli :as gp]))

(let [[graph output] (vec *command-line-args*)
      graph (path/resolve graph)
      config-file (path/join graph "logseq/config.edn")
      config (if (fs/existsSync config-file) (edn/read-string (str (fs/readFileSync config-file))) {})
      files (->> (common-graph/get-files graph)
                 (map #(hash-map :file/path %))
                 graph-parser/filter-files
                 (map #(update % :file/path (fn [file] (path/relative graph file))))
                 (#(common-config/remove-hidden-files % config :file/path))
                 (mapv #(assoc % :file/content (str (fs/readFileSync (path/join graph (:file/path %)))))))
      {:keys [conn]} (gp/parse-graph graph {:files files :verbose false})
      db @conn
      content-blocks (d/q '[:find [(pull ?e [*]) ...] :where [?e :block/content]] db)
      file-of (fn [e] (when-let [f (:block/file e)] (:file/path (d/entity db (:db/id f)))))
      label (fn [ref] (let [e (d/entity db (:db/id ref))]
                        (or (:block/name e) (some-> (:block/uuid e) str) "unresolved")))
      block-target? (fn [ref] (not (:block/name (d/entity db (:db/id ref)))))
      ;; Pages named only by an alias can lack a UUID, so pages are found by name.
      pages (->> (d/q '[:find [(pull ?e [*]) ...] :where [?e :block/name]] db)
                 (map (fn [e] {:name (:block/name e)
                               :original-name (:block/original-name e)
                               :file (file-of e)
                               :aliases (vec (sort (map label (:block/alias e))))
                               :namespace (when (:block/namespace e) (label (:block/namespace e)))
                               :properties (vec (sort (map name (keys (:block/properties e)))))
                               :journal? (boolean (:block/journal? e))
                               :journal-day (:block/journal-day e)}))
                 (sort-by :name)
                 vec)
      children (group-by #(get-in % [:block/parent :db/id]) content-blocks)
      ;; A block's first child has the block as its left neighbour; each later child has
      ;; its previous sibling.
      outline (fn outline [parent-id]
                (let [by-left (into {} (map (juxt #(get-in % [:block/left :db/id]) identity)
                                            (get children parent-id)))]
                  (loop [left parent-id acc []]
                    (if-let [child (get by-left left)]
                      (recur (:db/id child) (into (conj acc child) (outline (:db/id child))))
                      acc))))
      blocks (->> pages
                  (filter :file)
                  (mapcat (fn [{page-name :name file :file}]
                            (map-indexed
                             (fn [position e]
                               {:file file
                                :position position
                                :content (:block/content e)
                                :uuid (str (:block/uuid e))
                                :pre-block? (boolean (:block/pre-block? e))
                                :collapsed? (boolean (:block/collapsed? e))
                                :refs (vec (sort (map label (:block/refs e))))
                                :block-refs (vec (sort (map label (filter block-target? (:block/refs e)))))
                                :properties (vec (sort (map name (keys (:block/properties e)))))
                                :properties-text-values (:block/properties-text-values e)})
                             (outline (:db/id (d/entity db [:block/name page-name]))))))
                  vec)]
  (fs/writeFileSync output (js/JSON.stringify (clj->js {:pages pages :blocks blocks}) nil 2)))
