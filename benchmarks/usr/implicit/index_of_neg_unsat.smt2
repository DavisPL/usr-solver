(set-logic QF_S)

(declare-fun s () String)

(assert (not (= (str.indexof s "abc" 0) 2)))
(assert  (= (str.indexof s "c" 0) 4))
(assert  (= (str.indexof s "b" 0) 3))
(assert  (= (str.indexof s "a" 0) 2))

(check-sat)
(get-model)

