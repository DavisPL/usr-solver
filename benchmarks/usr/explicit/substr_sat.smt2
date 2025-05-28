(set-logic QF_S)

(declare-const s1 String)

(assert (= "pine" (str.substr s1 2 4)))
(assert (= "rine" (str.substr s1 8 13)))

(check-sat)

(get-model)

