(set-logic QF_S)

(declare-const s1 String)

(assert (not (= "pine" (str.substr s1 7 4))))
(assert (= "wine" (str.substr s1 8 13)))

(check-sat)

(get-model)

