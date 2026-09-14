(ns omniharness.patch-manager-test
  "Real behavior tests for omniharness.patch-manager: proposal lifecycle
   (submit -> approve/reject -> apply) and real file writes on apply.

   The kernel event store is a real gRPC service (see omniharness.events /
   omniharness.client) that isn't running in a unit-test environment, so we
   stub events/append-event! with with-redefs — the object under test here is
   the patch-manager's own state machine and file I/O, not the gRPC event
   pipeline (which belongs to a kernel-side test suite)."
  (:require [clojure.test :refer [deftest testing is use-fixtures]]
            [clojure.java.io :as io]
            [omniharness.events :as events]
            [omniharness.patch-manager :as pm]))

(defn- with-stubbed-events [f]
  (with-redefs [events/append-event! (fn [& _] {:success true :event-hash "stub"})]
    (f)))

(use-fixtures :each with-stubbed-events)

(deftest submit-proposal-creates-pending-proposal
  (let [p (pm/submit-proposal! "fix bug" "/tmp/target.clj" "(+ 1 1)")]
    (is (some? (:proposal-id p)))
    (is (= :pending (:status p)))
    (is (= "/tmp/target.clj" (:target-file p)))
    (is (= p (pm/get-proposal (:proposal-id p))))))

(deftest approve-proposal-transitions-status
  (let [p (pm/submit-proposal! "d" "/tmp/t.clj" "content")]
    (is (true? (pm/approve-proposal! (:proposal-id p))))
    (is (= :approved (:status (pm/get-proposal (:proposal-id p)))))
    (is (some? (:reviewed-at (pm/get-proposal (:proposal-id p)))))))

(deftest approve-unknown-proposal-returns-false
  (is (false? (pm/approve-proposal! "does-not-exist"))))

(deftest reject-proposal-records-reason
  (let [p (pm/submit-proposal! "d" "/tmp/t.clj" "content")]
    (is (true? (pm/reject-proposal! (:proposal-id p) "bad idea")))
    (let [reloaded (pm/get-proposal (:proposal-id p))]
      (is (= :rejected (:status reloaded)))
      (is (= "bad idea" (:rejection-reason reloaded))))))

(deftest apply-patch-requires-approval-first
  (let [p (pm/submit-proposal! "d" "/tmp/t.clj" "content")
        result (pm/apply-patch! (:proposal-id p))]
    (is (false? (:ok result)))
    (is (re-find #"not approved" (:error result)))
    ;; Applying an unapproved proposal must not change its status.
    (is (= :pending (:status (pm/get-proposal (:proposal-id p)))))))

(deftest apply-patch-unknown-proposal-returns-error
  (let [result (pm/apply-patch! "ghost-id")]
    (is (false? (:ok result)))
    (is (= "Proposal not found" (:error result)))))

(deftest apply-patch-writes-real-file-and-marks-applied
  (let [tmp (io/file (System/getProperty "java.io.tmpdir") (str "clj-orch-patch-" (System/nanoTime) ".txt"))
        path (.getAbsolutePath tmp)]
    (try
      (let [p (pm/submit-proposal! "write greeting" path "hello from patch manager")]
        (pm/approve-proposal! (:proposal-id p))
        (let [result (pm/apply-patch! (:proposal-id p))]
          (is (true? (:ok result)))
          (is (= path (:target result)))
          (is (= "hello from patch manager" (slurp path)))
          (is (= :applied (:status (pm/get-proposal (:proposal-id p)))))
          (is (some? (:applied-at (pm/get-proposal (:proposal-id p)))))))
      (finally (io/delete-file tmp true)))))

(deftest apply-patch-to-unwritable-path-returns-error-without-throwing
  (let [p (pm/submit-proposal! "bad path" "/nonexistent-dir-xyz/definitely/not/here.txt" "content")]
    (pm/approve-proposal! (:proposal-id p))
    (let [result (pm/apply-patch! (:proposal-id p))]
      (is (false? (:ok result)))
      (is (string? (:error result))))))

(deftest list-proposals-filters-by-status
  (let [a (pm/submit-proposal! "a" "/tmp/a.clj" "x")
        b (pm/submit-proposal! "b" "/tmp/b.clj" "y")]
    (pm/approve-proposal! (:proposal-id a))
    (let [pending  (set (map :proposal-id (pm/list-proposals :pending)))
          approved (set (map :proposal-id (pm/list-proposals :approved)))]
      (is (contains? approved (:proposal-id a)))
      (is (not (contains? pending (:proposal-id a))))
      (is (contains? pending (:proposal-id b))))))
