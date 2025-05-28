(set-logic QF_S)

(declare-const s1 String)

(assert 
 (= (str.at s1 0) "a")
 )

(assert 
 (= (str.at s1 1) "b")
)


(check-sat)

