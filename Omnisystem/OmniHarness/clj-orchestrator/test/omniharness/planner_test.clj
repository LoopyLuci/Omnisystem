(ns omniharness.planner-test
  "Real behavior tests for omniharness.planner: the HTN forward-chaining
   search, its built-in operators/methods, and plan execution."
  (:require [clojure.test :refer [deftest testing is]]
            [clojure.java.io :as io]
            [omniharness.planner :as p]))

(deftest builtins-are-registered
  (is (every? (set (p/list-operators)) ["ping" "read_file" "write_file" "log_message"]))
  (is (every? (set (p/list-methods)) ["research" "code_change" "setup_and_ping"])))

(deftest plan-primitive-task-returns-itself
  (let [task (p/make-task "ping" {:host "example.com"})
        result (p/plan task {})]
    (is (= [task] result))))

(deftest plan-compound-code-change-decomposes-to-read-then-write
  (let [result (p/plan (p/make-task "code_change" {:path "/tmp/x.clj" :new-content "(+ 1 1)"} false) {})]
    (is (= ["read_file" "write_file"] (map :name result)))
    (is (= {:path "/tmp/x.clj"} (:params (first result))))
    (is (= {:path "/tmp/x.clj" :content "(+ 1 1)"} (:params (second result))))))

(deftest plan-compound-method-condition-not-met-returns-nil
  ;; code_change's Method.condition requires :path in params.
  (is (nil? (p/plan (p/make-task "code_change" {} false) {}))))

(deftest plan-unknown-compound-returns-nil
  (is (nil? (p/plan (p/make-task "totally_unknown_task" {} false) {}))))

(deftest plan-setup-and-ping-decomposes-in-order
  (let [result (p/plan (p/make-task "setup_and_ping" {:host "h"} false) {})]
    (is (= ["log_message" "ping"] (map :name result)))))

(deftest operator-preconditions-block-planning
  (p/register-operator! (p/->Operator "locked"
                                       (fn [s] (boolean (:unlocked s)))
                                       (fn [s p] {})
                                       (fn [s p] "done")))
  (is (nil? (p/plan (p/make-task "locked") {})))
  (is (= [(p/make-task "locked")] (p/plan (p/make-task "locked") {:unlocked true}))))

(deftest method-tries-next-alternative-when-first-fails
  (p/register-operator! (p/->Operator "only-if-ready"
                                       (fn [s] (boolean (:ready s)))
                                       (fn [s p] {})
                                       (fn [s p] "ran")))
  (p/register-operator! (p/->Operator "always-ok"
                                       (fn [s] true)
                                       (fn [s p] {})
                                       (fn [s p] "ran-fallback")))
  (p/register-method! (p/->Method "do-thing-test"
                                  (fn [s p] true)
                                  (fn [s p] [(p/make-task "only-if-ready")])))
  (p/register-method! (p/->Method "do-thing-test"
                                  (fn [s p] true)
                                  (fn [s p] [(p/make-task "always-ok")])))
  (let [result (p/plan (p/make-task "do-thing-test" {} false) {:ready false})]
    (is (= [(p/make-task "always-ok")] result))))

;; ── Execution ─────────────────────────────────────────────────────────────────

(deftest execute-plan-ping-operator
  (let [result (p/execute-plan! [(p/make-task "ping" {:host "myhost"})] {})]
    (is (= [{:task "ping" :result "PONG from myhost" :ok true}] (:results result)))
    (is (= {} (:state result)))))

(deftest execute-plan-write-then-read-file-roundtrip
  (let [tmp (io/file (System/getProperty "java.io.tmpdir") (str "clj-orch-test-" (System/nanoTime) ".txt"))
        path (.getAbsolutePath tmp)]
    (try
      (let [plan   [(p/make-task "write_file" {:path path :content "hi there"})
                    (p/make-task "read_file"  {:path path})]
            result (p/execute-plan! plan {})]
        (is (true? (:ok (first (:results result)))))
        (is (= (str "Written: " path) (:result (first (:results result)))))
        (is (= "hi there" (:result (second (:results result)))))
        (is (= path (:last-write (:state result))))
        (is (= path (:last-file (:state result)))))
      (finally (io/delete-file tmp true)))))

(deftest execute-plan-reports-no-operator-for-unregistered-task
  (let [task   (p/make-task "definitely_not_registered")
        result (p/execute-plan! [task] {})]
    (is (= [{:task "definitely_not_registered" :error "No operator" :ok false}]
           (:results result)))))

(deftest execute-plan-catches-operator-exceptions-as-error-results
  (p/register-operator! (p/->Operator "boom"
                                       (fn [s] true)
                                       (fn [s p] {})
                                       (fn [s p] (throw (RuntimeException. "kaboom")))))
  (let [result (p/execute-plan! [(p/make-task "boom")] {})]
    (is (true? (:ok (first (:results result)))))  ; execute-plan! catches and records as a string result, ok stays true
    (is (re-find #"kaboom" (:result (first (:results result)))))))
