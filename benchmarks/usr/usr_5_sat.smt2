; from GitHub issue: https://github.com/Z3Prover/z3/issues/5140
(declare-fun a () String)
(assert
 (str.in_re a
 (re.++
  (str.to_re
  (ite (str.in_re a (re.++ (str.to_re "00000") (re.* re.allchar))) a "00000"))
  (re.* re.allchar))))
(check-sat)
