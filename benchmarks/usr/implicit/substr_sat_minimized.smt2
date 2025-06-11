(set-logic QF_S)

(declare-const s1 String)
(declare-const s2 String)

(assert (= "a" (str.substr s1 0 1)))
(assert (= "b" (str.substr s1 2 2)))

(check-sat)
