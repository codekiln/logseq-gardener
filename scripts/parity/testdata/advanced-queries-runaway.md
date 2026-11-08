- A transform that doubles every page forty times, which stops at the step limit. Only lsg builds this page.
- #+BEGIN_QUERY
  {:title [:h2 "Runaway"]
   :query [:find (pull ?p [*])
           :where
           [?p :block/name]]
   :result-transform (fn [rows] (let [a rows b0 (concat a a) b1 (concat b0 b0) b2 (concat b1 b1) b3 (concat b2 b2) b4 (concat b3 b3) b5 (concat b4 b4) b6 (concat b5 b5) b7 (concat b6 b6) b8 (concat b7 b7) b9 (concat b8 b8) b10 (concat b9 b9) b11 (concat b10 b10) b12 (concat b11 b11) b13 (concat b12 b12) b14 (concat b13 b13) b15 (concat b14 b14) b16 (concat b15 b15) b17 (concat b16 b16) b18 (concat b17 b17) b19 (concat b18 b18) b20 (concat b19 b19) b21 (concat b20 b20) b22 (concat b21 b21) b23 (concat b22 b22) b24 (concat b23 b23) b25 (concat b24 b24) b26 (concat b25 b25) b27 (concat b26 b26) b28 (concat b27 b27) b29 (concat b28 b28) b30 (concat b29 b29) b31 (concat b30 b30) b32 (concat b31 b31) b33 (concat b32 b32) b34 (concat b33 b33) b35 (concat b34 b34) b36 (concat b35 b35) b37 (concat b36 b36) b38 (concat b37 b37) b39 (concat b38 b38)] b39))}
  #+END_QUERY
