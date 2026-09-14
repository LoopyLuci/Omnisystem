(ns omniharness.governance-test
  "Real behavior tests for omniharness.governance: budgets, policy, the
   tamper-evident audit hash chain, and the kill switch."
  (:require [clojure.test :refer [deftest testing is]]
            [omniharness.governance :as gov]))

;; ── Audit hash chain ─────────────────────────────────────────────────────────

(deftest audit-chain-links-and-verifies
  (let [audit (gov/new-audit)]
    (gov/audit-append! audit "event_a" {:x 1})
    (gov/audit-append! audit "event_b" {:y 2})
    (let [events (:events @audit)]
      (is (= 2 (count events)))
      (is (= (:hash (first events)) (:prev (second events))))
      (is (true? (gov/audit-verify? audit))))))

(deftest audit-chain-detects-tampering
  (let [audit (gov/new-audit)]
    (gov/audit-append! audit "event_a" {:x 1})
    (gov/audit-append! audit "event_b" {:y 2})
    (swap! audit update-in [:events 0 :payload :x] (constantly 999))
    (is (false? (gov/audit-verify? audit)))))

(deftest empty-audit-log-verifies-true
  (is (true? (gov/audit-verify? (gov/new-audit)))))

;; ── Governor lifecycle ────────────────────────────────────────────────────────

(deftest new-governor-records-run-start-event
  (let [g (gov/new-governor)]
    (is (= 1 (count (:events @(:audit @g)))))
    (is (= "run_start" (:kind (first (:events @(:audit @g))))))))

(deftest checkpoint-throws-when-killed
  (let [g (gov/new-governor)]
    (gov/trip! g "user aborted")
    (is (thrown-with-msg? clojure.lang.ExceptionInfo #"user aborted"
                           (gov/checkpoint! g "step-1")))
    ;; The abort itself must be recorded in the audit trail.
    (is (= "aborted" (:kind (last (:events @(:audit @g))))))))

(deftest checkpoint-enforces-max-steps
  (let [g (gov/new-governor (assoc gov/default-budget :max-steps 2) gov/default-policy)]
    (gov/checkpoint! g "s1")
    (gov/checkpoint! g "s2")
    (is (thrown-with-msg? clojure.lang.ExceptionInfo #"max-steps exceeded"
                           (gov/checkpoint! g "s3")))))

(deftest check-model-enforces-allowlist
  (let [g (gov/new-governor gov/default-budget (assoc gov/default-policy :allowed-models #{"good"}))]
    (is (nil? (gov/check-model! g "good")))
    (is (thrown-with-msg? clojure.lang.ExceptionInfo #"model denied"
                           (gov/check-model! g "bad")))))

(deftest check-model-enforces-call-budget
  (let [g (gov/new-governor (assoc gov/default-budget :max-model-calls 1) gov/default-policy)]
    (gov/check-model! g "m")
    (gov/record-call! g "m" 100)
    (is (thrown-with-msg? clojure.lang.ExceptionInfo #"max-model-calls exceeded"
                           (gov/check-model! g "m")))))

(deftest check-tool-enforces-denylist-and-allowlist
  (let [g (gov/new-governor gov/default-budget (assoc gov/default-policy :denied-tools #{"rm_rf"}))]
    (is (nil? (gov/check-tool! g "read_file")))
    (is (thrown-with-msg? clojure.lang.ExceptionInfo #"tool denied"
                           (gov/check-tool! g "rm_rf"))))
  (let [g (gov/new-governor gov/default-budget (assoc gov/default-policy :allowed-tools #{"read_file"}))]
    (is (nil? (gov/check-tool! g "read_file")))
    (is (thrown-with-msg? clojure.lang.ExceptionInfo #"tool denied"
                           (gov/check-tool! g "write_file")))))

(deftest record-call-tracks-usage-and-enforces-token-budget
  (let [g (gov/new-governor (assoc gov/default-budget :max-tokens 1000) gov/default-policy)]
    (gov/record-call! g "m" 400)
    (is (= 1 (get-in @g [:usage :model-calls])))
    (is (= 400 (get-in @g [:usage :tokens])))
    (is (thrown-with-msg? clojure.lang.ExceptionInfo #"max-tokens exceeded"
                           (gov/record-call! g "m" 700)))))

(deftest record-call-enforces-cost-budget
  (let [g (gov/new-governor (assoc gov/default-budget :max-cost-usd 0.001 :max-tokens 0) gov/default-policy)]
    (is (thrown-with-msg? clojure.lang.ExceptionInfo #"max-cost-usd exceeded"
                           (gov/record-call! g "m" 1000)))))

(deftest parallelism-clamps-to-tightest-bound
  (let [g (gov/new-governor (assoc gov/default-budget :max-parallel 4)
                             (assoc gov/default-policy :max-agents 2))]
    (is (= 2 (gov/parallelism g 10)))
    (is (= 1 (gov/parallelism g 0)))))

(deftest report-reflects-governor-state
  (let [g (gov/new-governor)]
    (gov/checkpoint! g "s")
    (gov/record-call! g "m" 100)
    (let [r (gov/report g)]
      (is (= 1 (get-in r [:usage :model-calls])))
      (is (= 100 (get-in r [:usage :tokens])))
      (is (true? (:audit-valid r)))
      (is (false? (:killed r))))))
