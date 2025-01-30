; from GitHub issue: https://github.com/Z3Prover/z3/issues/5140
(declare-fun a () Bool)
(declare-fun b () Int)
(declare-fun c () String)
(declare-fun d () String)
(assert (= c (str.++ (str.replace d (str.substr (ite a c d) 0 b) c) d)))
(check-sat)
