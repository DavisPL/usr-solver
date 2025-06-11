; from GitHub issue: https://github.com/Z3Prover/z3/issues/5140
(declare-fun a () String)
(declare-fun b () String)
(assert (str.in_re a
     (re.diff (re.union (re.* (str.to_re "b")) (str.to_re "z")) (re.++ (re.* (str.to_re "z")) (str.to_re b)))))
(assert (= 0 (str.len b)))
(check-sat)
