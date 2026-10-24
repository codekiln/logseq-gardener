## Verification

The product change filters generated descendants in `Hierarchy::bottom_view` while leaving current-row construction, source classification, destination creation, index entries and path links unchanged.

The deep hierarchy unit fixture retains the generated current page and stored leaf as two rows through 128 prefixes, with all 129 destinations and every prefix link preserved. Twelve publication tests pass, including stored parents, generated current pages, private/alias gaps and identical unselected-versus-absent hierarchy output. Source contents remain unchanged.

Root independently reviewed the bounded product diff and its fixtures with no findings. Full local `mise run ci` passes, including release build, Rust and Node regressions, strict specification validation, RuleSync, docs and release checks. Root verified actual Gitpa current/stored rows, generated Asset and year collection navigation, keyboard collapse/reopen and namespace activation, mobile dark layout, continuous playback and all local destinations. Final consumer review is clear. PR133 merged after every core, platform and global check passed.
