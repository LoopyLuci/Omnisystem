(ns omniharness.policy-test
  "Real behavior tests for omniharness.policy: rule compilation, priority
   ordering, default-deny, the built-in default policy table, and the audit log."
  (:require [clojure.test :refer [deftest testing is]]
            [omniharness.policy :as policy]))

;; ── compile-policy / make-rule ──────────────────────────────────────────────

(deftest higher-priority-rule-wins
  (let [pol (policy/compile-policy
             [(policy/make-rule (fn [a _ _] (= a "x")) :deny 10)
              (policy/make-rule (fn [a _ _] (= a "x")) :allow 20)])]
    (is (= :allow (pol "x" {} nil)))))

(deftest no-matching-rule-defaults-to-deny
  (let [pol (policy/compile-policy
             [(policy/make-rule (fn [a _ _] (= a "known")) :allow 10)])]
    (is (= :deny (pol "unknown-action" {} nil)))))

(deftest compile-policy-accepts-plain-vector-rules
  (let [pol (policy/compile-policy
             [[(fn [a _ _] (= a "ping")) :allow]])]
    (is (= :allow (pol "ping" {} nil)))
    (is (= :deny (pol "other" {} nil)))))

;; ── built-in predicates ───────────────────────────────────────────────────────

(deftest action-is-predicate-matches-exact-action
  (let [pred (policy/action-is? "ping")]
    (is (true? (pred "ping" {} nil)))
    (is (false? (pred "pinger" {} nil)))))

(deftest action-prefix-predicate-matches-prefix
  (let [pred (policy/action-prefix? "memory.")]
    (is (true? (pred "memory.store" {} nil)))
    (is (false? (pred "shell.run" {} nil)))))

(deftest has-session-predicate
  (let [pred (policy/has-session?)]
    (is (false? (pred "any" {} nil)))
    (is (true? (pred "any" {} "sess-1")))))

(deftest arg-matches-predicate
  ;; arg-matches? is built on re-find, so a match returns the matched
  ;; substring (truthy, not literal `true`) and a miss returns nil.
  (let [pred (policy/arg-matches? :path "\\.txt$")]
    (is (= ".txt" (pred nil {:path "notes.txt"} nil)))
    (is (nil? (pred nil {:path "notes.md"} nil)))))

;; ── default-policy table ──────────────────────────────────────────────────────

(deftest default-policy-allows-safe-readonly-actions
  (is (policy/allowed? "ping" {}))
  (is (policy/allowed? "get_time" {}))
  (is (policy/allowed? "calculator" {}))
  (is (policy/allowed? "search_web" {}))
  (is (policy/allowed? "read_file" {})))

(deftest default-policy-requires-approval-for-mutating-actions
  (is (policy/requires-approval? "write_file" {}))
  (is (policy/requires-approval? "http_post" {})))

(deftest default-policy-sandboxes-shell-prefixed-actions
  (is (= :sandbox (policy/evaluate "shell.exec" {}))))

(deftest default-policy-denies-unknown-actions
  (is (= :deny (policy/evaluate "delete_everything" {})))
  (is (false? (policy/allowed? "delete_everything" {}))))

(deftest default-policy-memory-prefix-allowed
  (is (policy/allowed? "memory.retrieve" {})))

;; ── audit log ─────────────────────────────────────────────────────────────────

(deftest audit-decision-records-and-returns-result
  (policy/clear-audit-log!)
  (let [result (policy/audit-decision! "write_file" {:path "a.txt"} :ask-user "session-1")]
    (is (= :ask-user result))
    (let [log (policy/get-audit-log)]
      (is (= 1 (count log)))
      (is (= "write_file" (:action (first log))))
      (is (= "session-1" (:session-id (first log))))
      (is (= :ask-user (:result (first log)))))))

(deftest clear-audit-log-empties-it
  (policy/audit-decision! "ping" {} :allow nil)
  (policy/clear-audit-log!)
  (is (empty? (policy/get-audit-log))))
