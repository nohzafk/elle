(elle/epoch 12)
# A `match`-destructured `rest` alias is a BORROWED subview of the scrutinee
# (the `Rest` intrinsic loads the cdr pointer into the scrutinee's region pages,
# but the region solver only registers a counted container read for *call-site*
# `rest()`/`first()`, not for pattern loads). Passing such an alias as an
# owned-param CALL ARGUMENT (tail or not) makes the callee's param release free
# the caller's still-live scrutinee region — a use-after-free on the caller's
# original list.
#
# Witness: a self-recursive `match` walk passes `rest` to its own tail call;
# the caller's scratch list (built by `map`) must survive the walk intact.
# GREEN since the lowerer marks destructure-rest bindings borrowed
# (`destructure_alias_bindings` → the borrowed-arg incref at the call site).
# RED before the fix (the freed tail prints as <heap:...> / length fails).

(defn find-entry [entries name]
  (match entries
    () nil
    (entry & rest)
      (if (= (get entry :name) name) entry (find-entry rest name))
    _ nil))

(var i 0)
(while (%lt i 2000)
  (let* [r2 (map (fn [u] u) (list {:name "a"} {:name "b"} {:name "c"} {:name "d"}))
         found (find-entry r2 "c")]
    (assert (= (get found :name) "c") "find-entry returns the matching entry")
    (assert (= (length r2) 4) "the map-built list survives a tail-recursive match walk"))
  (assign i (%add i 1)))

(println "region-match-rest-tail-move-uaf: ok")
