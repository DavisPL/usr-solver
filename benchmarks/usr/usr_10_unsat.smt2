; from GitHub issue: https://github.com/Z3Prover/z3/issues/5140
(declare-fun a () String)
(assert
 (str.in_re a
  (re.diff re.allchar
   (re.++ (re.* re.allchar)
    (str.to_re (ite (str.in_re a re.allchar) a ""))))))
(check-sat)
