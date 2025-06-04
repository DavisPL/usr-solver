(set-logic QF_S)

(declare-fun s () String)

; Check if "abc" appears in s starting at index 2
(assert (= (str.indexof s "abc" 0) 2))

(check-sat)
(get-model)

