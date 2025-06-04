(set-logic QF_S)

(declare-const s1 String)

(assert (= "pine" (str.substr s1 8 5)))
(assert (= "rine" (str.substr s1 8 13)))

(check-sat)

